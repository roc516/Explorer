use explorer_app::ExplorerState;
use iced::window as iced_window;
use iced::widget::{container, text_input};
use iced::{alignment, Element, Fill};

use crate::fluent::{FONT_SIZE_ADDRESS, SPACE_MD};
use crate::message::{window as window_msg, Message as AppMessage};

use super::breadcrumbs::breadcrumb_bar;
use super::Message;

pub const ADDRESS_INPUT_ID: iced::widget::Id = iced::widget::Id::new("toolbar-address-input");

pub fn address_bar<'a>(
    editing: bool,
    address_input: &'a str,
    placeholder: String,
    model: &'a ExplorerState,
    window_id: iced_window::Id,
) -> Element<'a, AppMessage> {
    container(if editing {
        text_input(&placeholder, address_input)
            .id(ADDRESS_INPUT_ID)
            .on_input(move |value| {
                AppMessage::Window(
                    window_id,
                    window_msg::Message::Explorer(Message::AddressEdited(value)),
                )
            })
            .on_submit(AppMessage::Window(
                window_id,
                window_msg::Message::Explorer(Message::AddressSubmit),
            ))
            .size(FONT_SIZE_ADDRESS)
            .width(Fill)
            .into()
    } else {
        breadcrumb_bar(model, window_id)
    })
    .padding([0.0, SPACE_MD])
    .width(Fill)
    .height(Fill)
    .align_y(alignment::Vertical::Center)
    .into()
}
