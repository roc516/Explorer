mod address;
mod breadcrumbs;
mod icons;
mod message;
mod nav;

pub use address::ADDRESS_INPUT_ID;
pub use message::{Action, Message};

use explorer_core::EPath;
use explorer_app::{ids, ExplorerModel, LanguageBundle, ModelError, NavigationHistory};
use iced::window as iced_window;
use iced::widget::{container, row};
use iced::{alignment, Element, Fill, Task};
use iced::widget::operation::{focus, select_all};

use crate::fluent::{HEIGHT_COMMAND_BAR, PAGE_PADDING_H, SPACE_MD, SPACE_XS};
use crate::message::{window as window_msg, Message as AppMessage};

use address::address_bar;
use nav::nav_buttons;

pub struct Toolbar {
    navigation: NavigationHistory,
    address_input: String,
    address_editing: bool,
    reveal_path: Option<EPath>,
}

impl Toolbar {
    pub fn new(initial_path: &EPath) -> Self {
        Self {
            navigation: NavigationHistory::new(initial_path.clone()),
            address_input: initial_path.display(),
            address_editing: false,
            reveal_path: None,
        }
    }

    pub fn push_history(&mut self, path: EPath) {
        self.navigation.push(path);
    }

    pub fn is_address_editing(&self) -> bool {
        self.address_editing
    }

    pub fn cancel_address_edit(&mut self, current_path: &EPath) {
        self.address_editing = false;
        self.address_input = current_path.display();
    }

    /// Sync address bar after a directory finished loading.
    /// Returns a path that should be selected in the file list, if any.
    pub fn on_directory_loaded(&mut self, path: &EPath) -> Option<EPath> {
        self.address_editing = false;
        if let Some(reveal) = self.reveal_path.take() {
            self.address_input = reveal.display();
            Some(reveal)
        } else {
            self.address_input = path.display();
            None
        }
    }

    pub fn update(
        &mut self,
        message: Message,
        model: &mut ExplorerModel,
    ) -> (Task<window_msg::Message>, Option<Action>) {
        match message {
            Message::GoUp => {
                let action = model.go_up().map(|path| {
                    self.push_history(path.clone());
                    Action::Load(path)
                });
                (Task::none(), action)
            }
            Message::GoBack => {
                let action = self.navigation.go_back().map(|path| {
                    model.begin_load();
                    Action::Load(path)
                });
                (Task::none(), action)
            }
            Message::GoForward => {
                let action = self.navigation.go_forward().map(|path| {
                    model.begin_load();
                    Action::Load(path)
                });
                (Task::none(), action)
            }
            Message::Refresh => {
                let action = model.refresh().map(Action::Load);
                (Task::none(), action)
            }
            Message::AddressEdited(value) => {
                self.address_input = value;
                (Task::none(), None)
            }
            Message::AddressEditStart => {
                self.address_editing = true;
                self.address_input = model.current_path.display();
                (
                    focus::<window_msg::Message>(ADDRESS_INPUT_ID)
                        .chain(select_all(ADDRESS_INPUT_ID)),
                    None,
                )
            }
            Message::BreadcrumbNavigate(path) => {
                self.address_editing = false;
                let action = model.navigate(path).map(|path| {
                    self.push_history(path.clone());
                    Action::Load(path)
                });
                (Task::none(), action)
            }
            Message::AddressSubmit => (Task::none(), self.submit_address(model)),
        }
    }

    pub fn view(
        &self,
        bundle: LanguageBundle,
        current_path: &EPath,
        can_go_up: bool,
        window_id: iced_window::Id,
    ) -> Element<'_, AppMessage> {
        let address_placeholder = bundle.tr(ids::TOOLBAR_ADDRESS_PLACEHOLDER);

        container(
            row![
                nav_buttons(
                    self.navigation.can_go_back(),
                    self.navigation.can_go_forward(),
                    can_go_up,
                    window_id,
                ),
                address_bar(
                    self.address_editing,
                    &self.address_input,
                    address_placeholder,
                    current_path,
                    window_id,
                ),
            ]
            .spacing(SPACE_MD)
            .align_y(alignment::Vertical::Center)
            .width(Fill)
            .height(Fill),
        )
        .padding([SPACE_XS, PAGE_PADDING_H])
        .width(Fill)
        .height(HEIGHT_COMMAND_BAR)
        .into()
    }

    fn submit_address(&mut self, model: &mut ExplorerModel) -> Option<Action> {
        self.address_editing = false;
        let path = EPath::from_address(&self.address_input, &model.current_path);

        if !path.exists() {
            model.set_path_error(ModelError::InvalidPath);
            return None;
        }

        if path.is_directory() {
            self.reveal_path = None;
            return model.navigate(path).map(|path| {
                self.push_history(path.clone());
                Action::Load(path)
            });
        }

        if path.is_file() {
            let Some(parent) = path.parent() else {
                model.set_path_error(ModelError::InvalidPath);
                return None;
            };

            if parent == model.current_path {
                self.address_input = path.display();
                model.error = None;
                model.select_path(&path);
                model.status = explorer_app::StatusInfo::ItemCount(model.entries.len());
                return None;
            }

            self.reveal_path = Some(path);
            return model.navigate(parent).map(|path| {
                self.push_history(path.clone());
                Action::Load(path)
            });
        }

        model.set_path_error(ModelError::InvalidPath);
        None
    }
}

impl Default for Toolbar {
    fn default() -> Self {
        Self::new(&EPath::local(std::path::PathBuf::from("/")))
    }
}
