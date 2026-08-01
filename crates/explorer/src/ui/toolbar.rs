mod address;
mod breadcrumbs;
mod icons;
mod message;
mod nav;

pub use address::ADDRESS_INPUT_ID;
pub use message::{Action, Message};

use std::path::PathBuf;

use explorer_core::navigation_parent;
use explorer_app::{
    ids, AddressTarget, ExplorerState, LanguageBundle, ModelError, NavigationHistory,
};
use iced::window as iced_window;
use iced::widget::{container, row};
use iced::{alignment, Element, Fill, Task};
use iced::widget::operation::{focus, select_all};

use crate::fluent::{HEIGHT_COMMAND_BAR, PAGE_PADDING_H, SPACE_MD, SPACE_XS};
use crate::message::{window as window_msg, Message as AppMessage};

use address::address_bar;
use nav::nav_buttons;

pub struct ToolbarUi {
    navigation: NavigationHistory,
    address_input: String,
    address_editing: bool,
    reveal_path: Option<PathBuf>,
}

impl ToolbarUi {
    pub fn new(model: &ExplorerState) -> Self {
        Self {
            navigation: NavigationHistory::new(model.current_path().to_path_buf()),
            address_input: model.internal_display(),
            address_editing: false,
            reveal_path: None,
        }
    }
}

impl Default for ToolbarUi {
    fn default() -> Self {
        Self {
            navigation: NavigationHistory::new(PathBuf::from("/")),
            address_input: String::from("/"),
            address_editing: false,
            reveal_path: None,
        }
    }
}

pub fn push_history(ui: &mut ToolbarUi, path: PathBuf) {
    ui.navigation.push(path);
}

pub fn is_address_editing(ui: &ToolbarUi) -> bool {
    ui.address_editing
}

pub fn cancel_address_edit(ui: &mut ToolbarUi, model: &ExplorerState) {
    ui.address_editing = false;
    ui.address_input = model.internal_display();
}

/// Sync address bar after a directory finished loading.
/// Returns a navigation path that should be selected in the file list, if any.
pub fn on_directory_loaded(ui: &mut ToolbarUi, model: &ExplorerState) -> Option<PathBuf> {
    ui.address_editing = false;
    if let Some(reveal) = ui.reveal_path.take() {
        ui.address_input = reveal.display().to_string();
        Some(reveal)
    } else {
        ui.address_input = model.internal_display();
        None
    }
}

pub fn update(
    ui: &mut ToolbarUi,
    message: Message,
    model: &mut ExplorerState,
) -> (Task<window_msg::Message>, Option<Action>) {
    match message {
        Message::GoUp => {
            let action = model.go_up().map(|dir| {
                push_history(ui, dir.path().to_path_buf());
                Action::Load(dir)
            });
            (Task::none(), action)
        }
        Message::GoBack => {
            let action = ui.navigation.go_back().and_then(|path| {
                model.navigate(path).map(Action::Load)
            });
            (Task::none(), action)
        }
        Message::GoForward => {
            let action = ui.navigation.go_forward().and_then(|path| {
                model.navigate(path).map(Action::Load)
            });
            (Task::none(), action)
        }
        Message::Refresh => {
            let action = model.refresh().map(Action::Load);
            (Task::none(), action)
        }
        Message::AddressEdited(value) => {
            ui.address_input = value;
            (Task::none(), None)
        }
        Message::AddressEditStart => {
            ui.address_editing = true;
            ui.address_input = model.internal_display();
            (
                focus::<window_msg::Message>(ADDRESS_INPUT_ID)
                    .chain(select_all(ADDRESS_INPUT_ID)),
                None,
            )
        }
        Message::BreadcrumbNavigate(path) => {
            ui.address_editing = false;
            let action = model.navigate(path).map(|dir| {
                push_history(ui, dir.path().to_path_buf());
                Action::Load(dir)
            });
            (Task::none(), action)
        }
        Message::AddressSubmit => (Task::none(), submit_address(ui, model)),
    }
}

pub fn view<'a>(
    ui: &'a ToolbarUi,
    bundle: LanguageBundle,
    model: &'a ExplorerState,
    window_id: iced_window::Id,
) -> Element<'a, AppMessage> {
    let address_placeholder = bundle.tr(ids::TOOLBAR_ADDRESS_PLACEHOLDER);

    container(
        row![
            nav_buttons(
                ui.navigation.can_go_back(),
                ui.navigation.can_go_forward(),
                model.can_go_up(),
                window_id,
            ),
            address_bar(
                ui.address_editing,
                &ui.address_input,
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

fn submit_address(ui: &mut ToolbarUi, model: &mut ExplorerState) -> Option<Action> {
    ui.address_editing = false;

    match model.resolve_address(&ui.address_input) {
        Ok(AddressTarget::Directory(dir)) => {
            ui.reveal_path = None;
            let dir = model.navigate_dir(dir);
            push_history(ui, dir.path().to_path_buf());
            Some(Action::Load(dir))
        }
        Ok(AddressTarget::File { path }) => {
            let Some(parent_nav) =
                navigation_parent(&path, true)
            else {
                model.set_path_error(ModelError::InvalidPath);
                return None;
            };

            if parent_nav == model.current_path() {
                ui.address_input = path.display().to_string();
                model.file_list.error = None;
                model.select_path(&path);
                model.file_list.status = explorer_app::StatusInfo::ItemCount(model.file_list.entries.len());
                return None;
            }

            ui.reveal_path = Some(path);
            model.navigate(parent_nav).map(|dir| {
                push_history(ui, dir.path().to_path_buf());
                Action::Load(dir)
            })
        }
        Err(error) => {
            model.set_path_error(error);
            None
        }
    }
}
