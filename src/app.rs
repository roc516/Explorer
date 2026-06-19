use explorer_core::{detect_system_locale, ids, ExplorerModel, Language, LanguageBundle, Locale};
use iced::theme::Mode;
use iced::widget::{column, container, mouse_area, row, rule, stack};
use iced::{keyboard, system, Element, Fill, Subscription, Task, Theme};

use crate::message::{input, settings, theme, Message};
use crate::theme::AppTheme;
use crate::widget::directory_tree::{self, Action as TreeAction, DirectoryTreeWidget};
use crate::widget::file_list::{self, Action as FileListAction, FileListWidget};
use crate::widget::settings_dialog::{self, SettingsDialogWidget};
use crate::widget::status_bar::StatusBarWidget;
use crate::widget::toolbar::{self, ToolbarWidget};

pub struct App {
    pub model: ExplorerModel,
    pub toolbar: ToolbarWidget,
    pub directory_tree: DirectoryTreeWidget,
    pub file_list: FileListWidget,
    pub status_bar: StatusBarWidget,
    pub settings_dialog: SettingsDialogWidget,
    pub theme_choice: AppTheme,
    pub system_mode: Mode,
    pub language: Language,
    pub system_locale: Locale,
    pub settings_open: bool,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let system_locale = detect_system_locale();
        let language = Language::default();
        let mut model = ExplorerModel::new();
        model.set_locale(language.resolve(system_locale));
        let initial_path = model.current_path.clone();

        (
            Self {
                model,
                toolbar: ToolbarWidget::new(),
                directory_tree: DirectoryTreeWidget::new(),
                file_list: FileListWidget::new(),
                status_bar: StatusBarWidget::new(),
                settings_dialog: SettingsDialogWidget::new(),
                theme_choice: AppTheme::System,
                system_mode: Mode::default(),
                language,
                system_locale,
                settings_open: false,
            },
            Task::batch([
                file_list::load_directory_task(initial_path).map(Message::FileList),
                system::theme().map(|mode| Message::Theme(theme::Message::SystemChanged(mode))),
            ]),
        )
    }

    pub fn effective_locale(&self) -> Locale {
        self.language.resolve(self.system_locale)
    }

    pub fn bundle(&self) -> LanguageBundle {
        self.model.bundle
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Explorer(message) => self.update_explorer(message),
            Message::FileList(message) => self.update_file_list(message),
            Message::Tree(message) => self.update_tree(message),
            Message::Theme(message) => self.update_theme(message),
            Message::Locale(message) => self.update_locale(message),
            Message::Settings(message) => self.update_settings(message),
            Message::Input(message) => self.update_input(message),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let bundle = self.bundle();

        let main = column![
            self.toolbar.view(
                bundle,
                &self.model.address_input,
                self.model.can_go_back(),
                self.model.can_go_forward(),
                self.model.can_go_up(),
            ),
            rule::horizontal(1),
            row![
                self.directory_tree.view(bundle).map(Message::Tree),
                rule::vertical(1),
                self.file_list.view(&self.model).map(Message::FileList),
            ]
            .spacing(0)
            .width(Fill)
            .height(Fill),
            rule::horizontal(1),
            self.status_bar.view(&self.model),
        ]
        .width(Fill)
        .height(Fill);

        if self.settings_open {
            let overlay = mouse_area(
                container(
                    self.settings_dialog
                        .view(bundle, self.theme_choice, self.language),
                )
                .width(Fill)
                .height(Fill)
                .style(settings_dialog::backdrop),
            )
            .on_press(Message::Settings(settings::Message::Close));

            stack![main, overlay]
                .width(Fill)
                .height(Fill)
                .into()
        } else {
            main.into()
        }
    }

    pub fn subscription(_state: &Self) -> Subscription<Message> {
        Subscription::batch([
            keyboard::listen().filter_map(|event| {
                if let keyboard::Event::KeyPressed { key, modifiers, .. } = event {
                    Some(Message::Input(input::Message::KeyPressed(key, modifiers)))
                } else {
                    None
                }
            }),
            system::theme_changes()
                .map(|mode| Message::Theme(theme::Message::SystemChanged(mode))),
        ])
    }

    pub fn theme(state: &Self) -> Theme {
        state.theme_choice.resolve(state.system_mode)
    }

    pub fn title(&self) -> String {
        self.model.bundle.tr(ids::WINDOW_TITLE)
    }

    fn load_directory(&self, path: std::path::PathBuf) -> Task<Message> {
        file_list::load_directory_task(path).map(Message::FileList)
    }

    fn update_explorer(&mut self, message: toolbar::Message) -> Task<Message> {
        match message {
            toolbar::Message::GoUp => self
                .model
                .go_up()
                .map(|path| self.load_directory(path))
                .unwrap_or_else(Task::none),
            toolbar::Message::GoBack => self
                .model
                .go_back()
                .map(|path| self.load_directory(path))
                .unwrap_or_else(Task::none),
            toolbar::Message::GoForward => self
                .model
                .go_forward()
                .map(|path| self.load_directory(path))
                .unwrap_or_else(Task::none),
            toolbar::Message::Refresh => self
                .model
                .refresh()
                .map(|path| self.load_directory(path))
                .unwrap_or_else(Task::none),
            toolbar::Message::AddressEdited(value) => {
                self.model.set_address(value);
                Task::none()
            }
            toolbar::Message::AddressSubmit => self
                .model
                .submit_address()
                .map(|path| self.load_directory(path))
                .unwrap_or_else(Task::none),
        }
    }

    fn update_file_list(&mut self, message: file_list::Message) -> Task<Message> {
        let (task, action) = self.file_list.update(&mut self.model, message);
        let mut tasks = vec![task.map(Message::FileList)];

        if let Some(FileListAction::DirectoryChanged(path)) = action {
            tasks.push(self.directory_tree.sync_path(&path).map(Message::Tree));
        }

        Task::batch(tasks)
    }

    fn update_tree(&mut self, message: directory_tree::Message) -> Task<Message> {
        let (task, action) = self.directory_tree.update(message);
        let mut tasks = vec![task.map(Message::Tree)];

        if let Some(TreeAction::Navigate(path)) = action {
            if let Some(load_path) = self.model.navigate(path) {
                tasks.push(self.load_directory(load_path));
            }
        }

        Task::batch(tasks)
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
            settings::Message::PressInside => {}
        }
        Task::none()
    }

    fn update_locale(&mut self, message: settings_dialog::locale::Message) -> Task<Message> {
        let settings_dialog::locale::Message::Selected(language) = message;
        self.language = language;
        self.model.set_locale(self.effective_locale());
        Task::none()
    }

    fn update_input(&mut self, message: input::Message) -> Task<Message> {
        let input::Message::KeyPressed(key, modifiers) = message;

        if modifiers.control() {
            return Task::none();
        }

        match key {
            keyboard::Key::Named(keyboard::key::Named::Escape) if self.settings_open => {
                self.update_settings(settings::Message::Close)
            }
            keyboard::Key::Named(keyboard::key::Named::Enter) => {
                if let Some(index) = self.model.selected_index {
                    return self.update_file_list(file_list::Message::EntryDoubleClicked(index));
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
