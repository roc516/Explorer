use std::sync::Arc;

use explorer_core::BlockDevice;
use explorer_app::{ids, ExplorerState, Locale};
use iced::keyboard;
use iced::widget::{column, row, rule, stack};
use iced::window;
use iced::{Element, Fill, Task};

use crate::message::{input, window as window_msg, Message};
use crate::ui::directory_tree::{self, Action as TreeAction, TreeUi};
use crate::ui::file_list::{self, Action as FileListAction, FileListUi};
use crate::ui::preview::{
    self as preview_ui, document, hex, image, text, Message as PreviewMessage, PreviewState,
};
use crate::ui::status_bar;
use crate::ui::toolbar::{self, Action as ToolbarAction, ToolbarUi};

pub(crate) struct Explorer {
    pub(crate) model: ExplorerState,
    pub(crate) file_list_ui: FileListUi,
    pub(crate) tree_ui: TreeUi,
    pub(crate) toolbar_ui: ToolbarUi,
    pub(crate) preview: Option<PreviewState>,
}

impl Explorer {
    pub(crate) fn new_local(locale: Locale) -> Self {
        let mut model = ExplorerState::new_local();
        model.set_locale(locale);
        let toolbar_ui = ToolbarUi::new(&model);
        Self {
            model,
            toolbar_ui,
            tree_ui: TreeUi::new(),
            file_list_ui: FileListUi::new(),
            preview: None,
        }
    }

    pub(crate) fn new_mounted(device: BlockDevice, locale: Locale) -> Self {
        let mut model = ExplorerState::new_mounted(device);
        model.set_locale(locale);
        let toolbar_ui = ToolbarUi::new(&model);
        Self {
            model,
            toolbar_ui,
            tree_ui: TreeUi::new(),
            file_list_ui: FileListUi::new(),
            preview: None,
        }
    }

    pub(crate) fn view(&self, window_id: window::Id) -> Element<'_, Message> {
        let bundle = self.model.bundle.clone();
        let main = column![
            toolbar::view(
                &self.toolbar_ui,
                bundle,
                &self.model,
                window_id,
            ),
            rule::horizontal(1),
            row![
                directory_tree::view(&self.tree_ui, &self.model.tree_state, bundle)
                    .map(move |message| Message::Window(window_id, window_msg::Message::Tree(message))),
                rule::vertical(1),
                file_list::view(&self.file_list_ui, &self.model)
                    .map(move |message| {
                        Message::Window(window_id, window_msg::Message::FileList(message))
                    }),
            ]
            .spacing(0)
            .width(Fill)
            .height(Fill),
            rule::horizontal(1),
            status_bar::view(&self.model),
        ]
        .width(Fill)
        .height(Fill)
        .into();

        // Same as settings: keep Stack as root so opening preview does not rebuild main.
        let mut layers = vec![main];
        if let Some(preview_state) = &self.preview {
            layers.push(
                preview_ui::view(bundle, preview_state)
                    .map(move |message| {
                        Message::Window(window_id, window_msg::Message::Preview(message))
                    }),
            );
        }

        stack(layers).width(Fill).height(Fill).into()
    }

    fn load_directory(&self, dir: Arc<dyn explorer_core::DirEntry>) -> Task<window_msg::Message> {
        file_list::load_directory_from_dir(dir).map(window_msg::Message::FileList)
    }

    pub(crate) fn update_explorer(&mut self, message: toolbar::Message) -> Task<window_msg::Message> {
        let refresh_tree = matches!(message, toolbar::Message::Refresh);
        let (task, action) = toolbar::update(&mut self.toolbar_ui, message, &mut self.model);
        let mut tasks = vec![task];
        if let Some(ToolbarAction::Load(dir)) = action {
            tasks.push(self.load_directory(dir));
        }
        if refresh_tree {
            tasks.push(
                directory_tree::refresh(&mut self.tree_ui, &mut self.model.tree_state)
                    .map(window_msg::Message::Tree),
            );
        }
        Task::batch(tasks)
    }

    pub(crate) fn update_file_list(
        &mut self,
        message: file_list::Message,
    ) -> (Task<window_msg::Message>, Option<BlockDevice>) {
        let (task, action) = file_list::update(&mut self.file_list_ui, &mut self.model, message);
        let mut tasks = vec![task.map(window_msg::Message::FileList)];

        if let Some(action) = action {
            match action {
                FileListAction::Navigated(path) => {
                    toolbar::push_history(&mut self.toolbar_ui, path.clone());
                    tasks.push(
                        directory_tree::sync_path(&mut self.tree_ui, &mut self.model.tree_state, &path)
                            .map(window_msg::Message::Tree),
                    );
                }
                FileListAction::DirectoryLoaded(path) => {
                    if let Some(reveal) = toolbar::on_directory_loaded(&mut self.toolbar_ui, &self.model) {
                        self.model.select_path(&reveal);
                    }
                    tasks.push(
                        directory_tree::sync_path(&mut self.tree_ui, &mut self.model.tree_state, &path)
                            .map(window_msg::Message::Tree),
                    );
                }
                FileListAction::PreviewFile(entry) => {
                    tasks.push(self.open_preview(entry).map(window_msg::Message::Preview));
                }
                FileListAction::OpenArchive(device) => {
                    return (Task::batch(tasks), Some(device));
                }
            }
        }

        (Task::batch(tasks), None)
    }

    fn open_preview(&mut self, entry: explorer_core::FsEntry) -> Task<PreviewMessage> {
        self.preview = Some(PreviewState::opening(entry.clone()));
        preview_ui::load_preview_task(entry)
    }

    pub(crate) fn update_preview(&mut self, message: PreviewMessage) -> Task<window_msg::Message> {
        match message {
            PreviewMessage::Close => {
                self.preview = None;
            }
            PreviewMessage::Loaded(result) => {
                let bundle = self.model.bundle.clone();
                let task = if let Some(state) = &mut self.preview {
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
            PreviewMessage::OpenExternal => {
                if let Some(source) = self.preview.as_ref().map(|state| state.source.clone()) {
                    if let Err(message) = explorer_app::open_with_system(&source) {
                        if let Some(state) = &mut self.preview {
                            state.error = Some(message);
                        }
                    }
                }
            }
            PreviewMessage::Text(message) => {
                return self.update_text_preview(message);
            }
            PreviewMessage::Hex(message) => {
                return self.update_hex_preview(message);
            }
            PreviewMessage::Image(message) => {
                self.update_image_preview(message);
            }
            PreviewMessage::Document(message) => {
                self.update_document_preview(message);
            }
        }
        Task::none()
    }

    fn update_text_preview(&mut self, message: text::Message) -> Task<window_msg::Message> {
        let task = if let Some(state) = &mut self.preview {
            if let (Some(text_state), Some(file)) = (&mut state.text, state.file.as_ref()) {
                if let explorer_app::PreviewKind::Text(preview) = &file.kind {
                    let preview = preview.clone();
                    match message {
                        text::Message::Scrolled(y) => text_state.on_scroll(&preview, y),
                        text::Message::EncodingSelected(encoding) => {
                            text_state.select_encoding(&preview, encoding)
                        }
                        text::Message::IndexLoaded { id, result } => {
                            text_state.apply_index(&preview, id, result)
                        }
                        text::Message::WindowLoaded { id, start, result } => {
                            text_state.apply_window(id, start, result);
                            Task::none()
                        }
                    }
                    .map(PreviewMessage::Text)
                } else {
                    Task::none()
                }
            } else {
                Task::none()
            }
        } else {
            Task::none()
        };
        task.map(window_msg::Message::Preview)
    }

    fn update_hex_preview(&mut self, message: hex::Message) -> Task<window_msg::Message> {
        let task = if let Some(state) = &mut self.preview {
            if let (Some(hex_state), Some(file)) = (&mut state.hex, state.file.as_ref()) {
                if let explorer_app::PreviewKind::Hex(preview) = &file.kind {
                    let preview = preview.clone();
                    match message {
                        hex::Message::Scrolled(y) => hex_state.on_scroll(&preview, y),
                        hex::Message::Select(index) => {
                            hex_state.select(index);
                            Task::none()
                        }
                        hex::Message::WindowLoaded { id, start, result } => {
                            hex_state.apply_window(id, start, result);
                            Task::none()
                        }
                    }
                    .map(PreviewMessage::Hex)
                } else {
                    Task::none()
                }
            } else {
                Task::none()
            }
        } else {
            Task::none()
        };
        task.map(window_msg::Message::Preview)
    }

    fn update_image_preview(&mut self, message: image::Message) {
        if let Some(state) = &mut self.preview {
            if let Some(image_state) = &mut state.image {
                match message {
                    image::Message::ZoomIn => image_state.zoom_in(),
                    image::Message::ZoomOut => image_state.zoom_out(),
                    image::Message::ZoomReset => image_state.reset(),
                    image::Message::WheelZoom(factor) => image_state.wheel_zoom(factor),
                }
            }
        }
    }

    fn update_document_preview(&mut self, message: document::Message) {
        if let Some(state) = &mut self.preview {
            if let Some(document_state) = &mut state.document {
                match message {
                    document::Message::Editor(action) => {
                        document_state.handle_editor_action(action);
                    }
                }
            }
        }
    }

    pub(crate) fn update_tree(&mut self, message: directory_tree::Message) -> Task<window_msg::Message> {
        let (task, action) = directory_tree::update(&mut self.tree_ui, &mut self.model.tree_state, message);
        let mut tasks = vec![task.map(window_msg::Message::Tree)];

        if let Some(TreeAction::Navigate(dir)) = action {
            let dir = self.model.navigate_dir(dir);
            toolbar::push_history(&mut self.toolbar_ui, dir.path().to_path_buf());
            tasks.push(self.load_directory(dir));
        }

        Task::batch(tasks)
    }

    pub(crate) fn update_input(
        &mut self,
        message: input::Message,
        settings_open: bool,
    ) -> Task<window_msg::Message> {
        let input::Message::KeyPressed(key, modifiers) = message;

        if modifiers.control() {
            return Task::none();
        }

        match key {
            keyboard::Key::Named(keyboard::key::Named::Escape) if self.preview.is_some() => {
                self.update_preview(PreviewMessage::Close)
            }
            keyboard::Key::Named(keyboard::key::Named::Escape) if settings_open => Task::none(),
            keyboard::Key::Named(keyboard::key::Named::Escape)
                if toolbar::is_address_editing(&self.toolbar_ui) =>
            {
                toolbar::cancel_address_edit(&mut self.toolbar_ui, &self.model);
                Task::none()
            }
            _ if self.preview.is_some() || settings_open => Task::none(),
            keyboard::Key::Named(keyboard::key::Named::Enter) => {
                if let Some(index) = self.model.file_list.selected_index {
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
