use std::collections::BTreeMap;

use explorer_core::BlockDevice;
use explorer_app::{
    detect_system_locale, ids, ExplorerModel, Language, Locale,
};
use iced::keyboard;
use iced::theme::Mode;
use iced::widget::{column, row, rule, stack};
use iced::window;
use iced::window::settings::PlatformSpecific;
use iced::{Element, Fill, Subscription, Task, Theme};

use crate::message::{input, preview, settings, theme, window as window_msg, Message, Launch};
use crate::theme::AppTheme;
use crate::ui::directory_tree::{self, Action as TreeAction, DirectoryTree};
use crate::ui::file_list::{self, Action as FileListAction, FileList};
use crate::ui::preview::{self as preview_ui, Preview, PreviewState};
use crate::ui::settings::{self as settings_ui, Settings};
use crate::ui::status_bar::StatusBar;
use crate::ui::toolbar::{self, Action as ToolbarAction, Toolbar};

const WINDOW_WIDTH: f32 = 1200.0;
const WINDOW_HEIGHT: f32 = 760.0;

pub struct App {
    windows: BTreeMap<window::Id, Explorer>,
    focused_window: Option<window::Id>,
    settings: Settings,
    settings_open: bool,
    theme_choice: AppTheme,
    system_mode: Mode,
    language: Language,
    system_locale: Locale,
}

struct Explorer {
    model: ExplorerModel,
    toolbar: Toolbar,
    directory_tree: DirectoryTree,
    file_list: FileList,
    status_bar: StatusBar,
    preview: Preview,
    preview_state: Option<PreviewState>,
}

impl App {
    pub fn boot() -> (Self, Task<Message>) {
        explorer_core::ensure_host_registered(Box::new(explorer_fs_folder::FolderBackend));
        explorer_core::ensure_backends_registered(|registry| {
            registry.register(Box::new(explorer_fs_zip::ZipBackend));
        });

        let system_locale = detect_system_locale();
        let language = Language::default();
        let (_, open) = window::open(window_settings());

        (
            Self {
                windows: BTreeMap::new(),
                focused_window: None,
                settings: Settings::new(),
                settings_open: false,
                theme_choice: AppTheme::System,
                system_mode: Mode::default(),
                language,
                system_locale,
            },
            open.map(|id| Message::WindowOpened(id, Launch::Local)),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Window(id, message) => self.update_window(id, message),
            Message::WindowOpened(id, launch) => self.on_window_opened(id, launch),
            Message::WindowClosed(id) => self.on_window_closed(id),
            Message::WindowFocused(id) => {
                self.focused_window = Some(id);
                Task::none()
            }
            Message::Theme(message) => self.update_theme(message),
            Message::Locale(message) => self.update_locale(message),
            Message::Settings(message) => self.update_settings(message),
        }
    }

    pub fn view(&self, window_id: window::Id) -> Element<'_, Message> {
        let Some(window) = self.windows.get(&window_id) else {
            return iced::widget::Space::new().into();
        };

        // Keep Stack as the stable root. Toggling settings must only add/remove the
        // overlay child — switching root from Column to Stack rebuilds the whole UI
        // tree (file list + directory tree) and feels like a hitch on open.
        let mut layers = vec![window.view(window_id)];
        if self.settings_open {
            layers.push(self.settings.view(
                window.model.bundle,
                self.theme_choice,
                self.language,
            ));
        }

        stack(layers).width(Fill).height(Fill).into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let mut subscriptions = vec![
            window::close_events().map(Message::WindowClosed),
            window::events().filter_map(|(id, event)| {
                (event == window::Event::Focused).then_some(Message::WindowFocused(id))
            }),
            iced::system::theme_changes()
                .map(|mode| Message::Theme(theme::Message::SystemChanged(mode))),
        ];

        subscriptions.push(
            keyboard::listen()
                .with(self.focused_window)
                .filter_map(|(focused, event)| {
                    let id = focused?;
                    if let keyboard::Event::KeyPressed { key, modifiers, .. } = event {
                        Some(Message::Window(
                            id,
                            window_msg::Message::Input(input::Message::KeyPressed(key, modifiers)),
                        ))
                    } else {
                        None
                    }
                }),
        );

        let file_list_subscriptions: Vec<_> = self
            .windows
            .iter()
            .map(|(id, window)| {
                (
                    *id,
                    window.file_list.subscription(),
                )
            })
            .collect();

        for (id, subscription) in file_list_subscriptions {
            subscriptions.push(
                subscription
                    .with(id)
                    .map(|(id, message)| Message::Window(id, window_msg::Message::FileList(message))),
            );
        }

        Subscription::batch(subscriptions)
    }

    pub fn theme(&self, _window: window::Id) -> Option<Theme> {
        Some(self.theme_choice.resolve(self.system_mode))
    }

    pub fn title(&self, window_id: window::Id) -> String {
        self.windows
            .get(&window_id)
            .map(|window| {
                let title = window.model.bundle.tr(ids::WINDOW_TITLE);
                format!("{title} — {}", window.model.display_path())
            })
            .unwrap_or_else(|| "Explorer".to_string())
    }

    fn on_window_opened(&mut self, id: window::Id, launch: Launch) -> Task<Message> {
        let locale = self.language.resolve(self.system_locale);
        let explorer = match launch {
            Launch::Local => Explorer::new_local(locale),
            Launch::Archive(device) => Explorer::new_mounted(device, locale),
        };

        let load_dir = explorer.model.current_dir().clone();
        self.focused_window = Some(id);
        self.windows.insert(id, explorer);

        file_list::load_directory_from_dir(load_dir)
            .map(move |message| Message::Window(id, window_msg::Message::FileList(message)))
    }

    fn on_window_closed(&mut self, id: window::Id) -> Task<Message> {
        self.windows.remove(&id);
        if self.focused_window == Some(id) {
            self.focused_window = self.windows.keys().next().copied();
        }

        if self.windows.is_empty() {
            iced::exit()
        } else {
            Task::none()
        }
    }

    fn open_mounted_window(&self, device: BlockDevice) -> Task<Message> {
        let (_, open) = window::open(window_settings());
        open.map(move |id| Message::WindowOpened(id, Launch::Archive(device.clone())))
    }

    fn update_window(&mut self, id: window::Id, message: window_msg::Message) -> Task<Message> {
        let Some(window) = self.windows.get_mut(&id) else {
            return Task::none();
        };

        let task = match message {
            window_msg::Message::Explorer(message) => window.update_explorer(message),
            window_msg::Message::FileList(message) => {
                let (task, _open_mounted) = window.update_file_list(message);
                if let Some(container) = _open_mounted {
                    return Task::batch([
                        task.map(move |message| Message::Window(id, message)),
                        self.open_mounted_window(container),
                    ]);
                }
                task
            }
            window_msg::Message::Tree(message) => window.update_tree(message),
            window_msg::Message::Preview(message) => window.update_preview(message),
            window_msg::Message::Input(message) => {
                window.update_input(message, self.settings_open)
            }
        };

        task.map(move |message| Message::Window(id, message))
    }

    fn update_theme(&mut self, message: theme::Message) -> Task<Message> {
        match message {
            theme::Message::Selected(choice) => {
                self.theme_choice = choice;
            }
            theme::Message::SystemChanged(mode) => {
                self.system_mode = mode;
            }
        }
        Task::none()
    }

    fn update_settings(&mut self, message: settings::Message) -> Task<Message> {
        match message {
            settings::Message::Toggle => {
                self.settings_open = !self.settings_open;
            }
            settings::Message::Close => {
                self.settings_open = false;
            }
        }
        Task::none()
    }

    fn update_locale(&mut self, message: settings_ui::locale::Message) -> Task<Message> {
        let settings_ui::locale::Message::Selected(language) = message;
        self.language = language;
        let locale = self.language.resolve(self.system_locale);
        for window in self.windows.values_mut() {
            window.model.set_locale(locale);
        }
        Task::none()
    }
}

impl Explorer {
    fn new_local(locale: Locale) -> Self {
        let mut model = ExplorerModel::new_local();
        model.set_locale(locale);
        let toolbar = Toolbar::new(&model);
        Self {
            model,
            toolbar,
            directory_tree: DirectoryTree::new(),
            file_list: FileList::new(),
            status_bar: StatusBar::new(),
            preview: Preview::new(),
            preview_state: None,
        }
    }

    fn new_mounted(device: BlockDevice, locale: Locale) -> Self {
        let mut model = ExplorerModel::new_mounted(device.clone());
        model.set_locale(locale);
        let toolbar = Toolbar::new(&model);
        Self {
            model,
            toolbar,
            directory_tree: DirectoryTree::for_mounted(device),
            file_list: FileList::new(),
            status_bar: StatusBar::new(),
            preview: Preview::new(),
            preview_state: None,
        }
    }

    fn view(&self, window_id: window::Id) -> Element<'_, Message> {
        let bundle = self.model.bundle.clone();
        let main = column![
            self.toolbar.view(
                bundle,
                &self.model,
                window_id,
            ),
            rule::horizontal(1),
            row![
                self.directory_tree
                    .view(bundle)
                    .map(move |message| Message::Window(window_id, window_msg::Message::Tree(message))),
                rule::vertical(1),
                self.file_list
                    .view(&self.model)
                    .map(move |message| {
                        Message::Window(window_id, window_msg::Message::FileList(message))
                    }),
            ]
            .spacing(0)
            .width(Fill)
            .height(Fill),
            rule::horizontal(1),
            self.status_bar.view(&self.model),
        ]
        .width(Fill)
        .height(Fill)
        .into();

        // Same as settings: keep Stack as root so opening preview does not rebuild main.
        let mut layers = vec![main];
        if let Some(preview_state) = &self.preview_state {
            layers.push(
                self.preview
                    .view(bundle, preview_state)
                    .map(move |message| {
                        Message::Window(window_id, window_msg::Message::Preview(message))
                    }),
            );
        }

        stack(layers).width(Fill).height(Fill).into()
    }

    fn load_directory(&self, dir: explorer_core::DirEntry) -> Task<window_msg::Message> {
        file_list::load_directory_from_dir(dir).map(window_msg::Message::FileList)
    }

    fn update_explorer(&mut self, message: toolbar::Message) -> Task<window_msg::Message> {
        let (task, action) = self.toolbar.update(message, &mut self.model);
        let mut tasks = vec![task];
        if let Some(ToolbarAction::Load(dir)) = action {
            tasks.push(self.load_directory(dir));
        }
        Task::batch(tasks)
    }

    fn update_file_list(
        &mut self,
        message: file_list::Message,
    ) -> (Task<window_msg::Message>, Option<BlockDevice>) {
        let (task, action) = self.file_list.update(&mut self.model, message);
        let mut tasks = vec![task.map(window_msg::Message::FileList)];

        if let Some(action) = action {
            match action {
                FileListAction::Navigated(path) => {
                    self.toolbar.push_history(path.clone());
                    tasks.push(
                        self.directory_tree
                            .sync_path(&path)
                            .map(window_msg::Message::Tree),
                    );
                }
                FileListAction::DirectoryLoaded(path) => {
                    if let Some(reveal) = self.toolbar.on_directory_loaded(&self.model) {
                        self.model.select_path(&reveal);
                    }
                    tasks.push(
                        self.directory_tree
                            .sync_path(&path)
                            .map(window_msg::Message::Tree),
                    );
                }
                FileListAction::PreviewFile(entry) => {
                    tasks.push(self.open_preview(entry).map(window_msg::Message::Preview));
                }
                FileListAction::OpenArchive(device) => {
                    return (Task::none(), Some(device));
                }
            }
        }

        (Task::batch(tasks), None)
    }

    fn open_preview(&mut self, entry: explorer_core::FsEntry) -> Task<preview::Message> {
        self.preview_state = Some(PreviewState::opening(entry.clone()));
        preview_ui::load_preview_task(entry)
    }

    fn update_preview(&mut self, message: preview::Message) -> Task<window_msg::Message> {
        match message {
            preview::Message::Close => {
                self.preview_state = None;
            }
            preview::Message::Loaded(result) => {
                let bundle = self.model.bundle.clone();
                let task = if let Some(state) = &mut self.preview_state {
                    state.loading = false;
                    match result {
                        Ok(file) => state.set_loaded_file(file),
                        Err(code) => {
                            state.error = Some(match code.as_str() {
                                "preview-too-large" => bundle.tr(ids::PREVIEW_TOO_LARGE),
                                "preview-decode-failed" => bundle.tr(ids::PREVIEW_DECODE_FAILED),
                                "preview-not-file" => bundle.tr(ids::PREVIEW_NOT_FILE),
                                "preview-word-failed" => bundle.tr(ids::PREVIEW_WORD_FAILED),
                                "preview-ppt-failed" => bundle.tr(ids::PREVIEW_PPT_FAILED),
                                "preview-pdf-failed" => bundle.tr(ids::PREVIEW_PDF_FAILED),
                                _ => bundle.tr(ids::PREVIEW_LOAD_FAILED),
                            });
                            Task::none()
                        }
                    }
                } else {
                    Task::none()
                };
                return task.map(window_msg::Message::Preview);
            }
            preview::Message::OpenExternal => {
                if let Some(source) = self.preview_state.as_ref().map(|state| state.source.clone()) {
                    if let Err(message) = explorer_app::open_with_system(&source) {
                        if let Some(state) = &mut self.preview_state {
                            state.error = Some(message);
                        }
                    }
                }
            }
            preview::Message::EncodingSelected(encoding) => {
                let bundle = self.model.bundle.clone();
                if let Some(state) = &mut self.preview_state {
                    if let (Some(text), Some(file)) = (&mut state.text, &mut state.file) {
                        text.select_encoding(
                            file,
                            encoding,
                            || bundle.tr(ids::PREVIEW_DECODE_FAILED),
                            || bundle.tr(ids::PREVIEW_LOAD_FAILED),
                        );
                    }
                }
            }
            preview::Message::TextEditor(action) => {
                if let Some(state) = &mut self.preview_state {
                    if let Some(text) = &mut state.text {
                        text.handle_editor_action(action);
                    }
                }
            }
            preview::Message::DocumentEditor(action) => {
                if let Some(state) = &mut self.preview_state {
                    if let Some(document) = &mut state.document {
                        document.handle_editor_action(action);
                    }
                }
            }
            preview::Message::ImageZoomIn => {
                if let Some(state) = &mut self.preview_state {
                    if let Some(image) = &mut state.image {
                        image.zoom_in();
                    }
                }
            }
            preview::Message::ImageZoomOut => {
                if let Some(state) = &mut self.preview_state {
                    if let Some(image) = &mut state.image {
                        image.zoom_out();
                    }
                }
            }
            preview::Message::ImageZoomReset => {
                if let Some(state) = &mut self.preview_state {
                    if let Some(image) = &mut state.image {
                        image.reset();
                    }
                }
            }
            preview::Message::ImageWheelZoom(factor) => {
                if let Some(state) = &mut self.preview_state {
                    if let Some(image) = &mut state.image {
                        image.wheel_zoom(factor);
                    }
                }
            }
            preview::Message::HexScrolled(y) => {
                let task = if let Some(state) = &mut self.preview_state {
                    if let (Some(hex), Some(file)) = (&mut state.hex, state.file.as_ref()) {
                        if let explorer_app::PreviewKind::Hex(preview) = &file.kind {
                            let preview = preview.clone();
                            hex.on_scroll(&preview, y)
                        } else {
                            Task::none()
                        }
                    } else {
                        Task::none()
                    }
                } else {
                    Task::none()
                };
                return task.map(window_msg::Message::Preview);
            }
            preview::Message::HexSelect(index) => {
                if let Some(state) = &mut self.preview_state {
                    if let Some(hex) = &mut state.hex {
                        hex.select(index);
                    }
                }
            }
            preview::Message::HexWindowLoaded { id, start, result } => {
                if let Some(state) = &mut self.preview_state {
                    if let Some(hex) = &mut state.hex {
                        hex.apply_window(id, start, result);
                    }
                }
            }
        }
        Task::none()
    }

    fn update_tree(&mut self, message: directory_tree::Message) -> Task<window_msg::Message> {
        let (task, action) = self.directory_tree.update(message);
        let mut tasks = vec![task.map(window_msg::Message::Tree)];

        if let Some(TreeAction::Navigate(dir)) = action {
            let dir = self.model.navigate_dir(dir);
            self.toolbar.push_history(dir.path.clone());
            tasks.push(self.load_directory(dir));
        }

        Task::batch(tasks)
    }

    fn update_input(
        &mut self,
        message: input::Message,
        settings_open: bool,
    ) -> Task<window_msg::Message> {
        let input::Message::KeyPressed(key, modifiers) = message;

        if modifiers.control() {
            return Task::none();
        }

        match key {
            keyboard::Key::Named(keyboard::key::Named::Escape) if self.preview_state.is_some() => {
                self.update_preview(preview::Message::Close)
            }
            keyboard::Key::Named(keyboard::key::Named::Escape) if settings_open => Task::none(),
            keyboard::Key::Named(keyboard::key::Named::Escape)
                if self.toolbar.is_address_editing() =>
            {
                self.toolbar
                    .cancel_address_edit(&self.model);
                Task::none()
            }
            _ if self.preview_state.is_some() || settings_open => Task::none(),
            keyboard::Key::Named(keyboard::key::Named::Enter) => {
                if let Some(index) = self.model.selected_index {
                    let (task, _) =
                        self.update_file_list(file_list::Message::EntryDoubleClicked(index));
                    return task;
                }
                Task::none()
            }
            keyboard::Key::Named(keyboard::key::Named::Backspace) => {
                self.update_explorer(toolbar::Message::GoUp)
            }
            keyboard::Key::Named(keyboard::key::Named::F5) => {
                self.update_explorer(toolbar::Message::Refresh)
            }
            keyboard::Key::Named(keyboard::key::Named::ArrowLeft) => {
                self.update_explorer(toolbar::Message::GoBack)
            }
            keyboard::Key::Named(keyboard::key::Named::ArrowRight) => {
                self.update_explorer(toolbar::Message::GoForward)
            }
            _ => Task::none(),
        }
    }
}

fn window_settings() -> window::Settings {
    window::Settings {
        size: iced::Size::new(WINDOW_WIDTH, WINDOW_HEIGHT),
        min_size: Some(iced::Size::new(800.0, 500.0)),
        icon: Some(crate::window_icon::app_icon()),
        #[cfg(target_os = "linux")]
        platform_specific: PlatformSpecific {
            application_id: "org.explorer.app".to_string(),
            ..Default::default()
        },
        ..Default::default()
    }
}
