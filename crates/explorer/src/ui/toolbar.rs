mod address;
mod breadcrumbs;
mod icons;
mod message;
mod nav;

pub use address::ADDRESS_INPUT_ID;
pub use message::{Action, Message};

use std::path::PathBuf;

use explorer_core::{navigation_parent, EPath, Mounter};
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
    reveal_path: Option<PathBuf>,
}

impl Toolbar {
    pub fn new(model: &ExplorerModel) -> Self {
        Self {
            navigation: NavigationHistory::new(model.current_path().to_path_buf()),
            address_input: model.internal_display(),
            address_editing: false,
            reveal_path: None,
        }
    }

    pub fn push_history(&mut self, path: PathBuf) {
        self.navigation.push(path);
    }

    pub fn is_address_editing(&self) -> bool {
        self.address_editing
    }

    pub fn cancel_address_edit(&mut self, model: &ExplorerModel) {
        self.address_editing = false;
        self.address_input = model.internal_display();
    }

    /// Sync address bar after a directory finished loading.
    /// Returns a navigation path that should be selected in the file list, if any.
    pub fn on_directory_loaded(&mut self, model: &ExplorerModel) -> Option<PathBuf> {
        self.address_editing = false;
        if let Some(reveal) = self.reveal_path.take() {
            self.address_input = reveal.display().to_string();
            Some(reveal)
        } else {
            self.address_input = model.internal_display();
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
                let action = model.go_up().map(|dir| {
                    self.push_history(dir.path.clone());
                    Action::Load(dir)
                });
                (Task::none(), action)
            }
            Message::GoBack => {
                let action = self.navigation.go_back().and_then(|path| {
                    model.navigate(path).map(Action::Load)
                });
                (Task::none(), action)
            }
            Message::GoForward => {
                let action = self.navigation.go_forward().and_then(|path| {
                    model.navigate(path).map(Action::Load)
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
                self.address_input = model.internal_display();
                (
                    focus::<window_msg::Message>(ADDRESS_INPUT_ID)
                        .chain(select_all(ADDRESS_INPUT_ID)),
                    None,
                )
            }
            Message::BreadcrumbNavigate(path) => {
                self.address_editing = false;
                let action = model.navigate(path).map(|dir| {
                    self.push_history(dir.path.clone());
                    Action::Load(dir)
                });
                (Task::none(), action)
            }
            Message::AddressSubmit => (Task::none(), self.submit_address(model)),
        }
    }

    pub fn view<'a>(
        &'a self,
        bundle: LanguageBundle,
        model: &'a ExplorerModel,
        window_id: iced_window::Id,
    ) -> Element<'a, AppMessage> {
        let address_placeholder = bundle.tr(ids::TOOLBAR_ADDRESS_PLACEHOLDER);

        container(
            row![
                nav_buttons(
                    self.navigation.can_go_back(),
                    self.navigation.can_go_forward(),
                    model.can_go_up(),
                    window_id,
                ),
                address_bar(
                    self.address_editing,
                    &self.address_input,
                    address_placeholder,
                    model,
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
        let location = model.location();
        let path = EPath::from_address(&self.address_input, &location);

        if !path.exists() {
            model.set_path_error(ModelError::InvalidPath);
            return None;
        }

        if path.is_directory() {
            self.reveal_path = None;
            return model.navigate(path.navigation_path()).map(|dir| {
                self.push_history(dir.path.clone());
                Action::Load(dir)
            });
        }

        if path.is_file() {
            let nav = path.navigation_path();
            let Some(parent_nav) = navigation_parent(&nav, Mounter::is_mount(&path)) else {
                model.set_path_error(ModelError::InvalidPath);
                return None;
            };

            if parent_nav == model.current_path() {
                self.address_input = path.internal_display();
                model.error = None;
                model.select_path(path.path());
                model.status = explorer_app::StatusInfo::ItemCount(model.entries.len());
                return None;
            }

            self.reveal_path = Some(nav);
            return model.navigate(parent_nav).map(|dir| {
                self.push_history(dir.path.clone());
                Action::Load(dir)
            });
        }

        model.set_path_error(ModelError::InvalidPath);
        None
    }
}

impl Default for Toolbar {
    fn default() -> Self {
        Self {
            navigation: NavigationHistory::new(PathBuf::from("/")),
            address_input: String::from("/"),
            address_editing: false,
            reveal_path: None,
        }
    }
}
