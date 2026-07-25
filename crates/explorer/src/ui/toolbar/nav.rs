use iced::window as iced_window;
use iced::widget::row;
use iced::Element;

use crate::fluent::SPACE_XS;
use crate::message::{settings, window as window_msg, Message as AppMessage};

use super::icons::{self as toolbar_icons, NavIcon};
use super::Message;

pub fn nav_buttons(
    can_go_back: bool,
    can_go_forward: bool,
    can_go_up: bool,
    window_id: iced_window::Id,
) -> Element<'static, AppMessage> {
    row![
        toolbar_icons::nav_button(
            NavIcon::Back,
            can_go_back,
            can_go_back.then_some(AppMessage::Window(
                window_id,
                window_msg::Message::Explorer(Message::GoBack),
            )),
        ),
        toolbar_icons::nav_button(
            NavIcon::Forward,
            can_go_forward,
            can_go_forward.then_some(AppMessage::Window(
                window_id,
                window_msg::Message::Explorer(Message::GoForward),
            )),
        ),
        toolbar_icons::nav_button(
            NavIcon::Up,
            can_go_up,
            can_go_up.then_some(AppMessage::Window(
                window_id,
                window_msg::Message::Explorer(Message::GoUp),
            )),
        ),
        toolbar_icons::nav_button(
            NavIcon::Refresh,
            true,
            Some(AppMessage::Window(
                window_id,
                window_msg::Message::Explorer(Message::Refresh),
            )),
        ),
        toolbar_icons::nav_button(
            NavIcon::Settings,
            true,
            Some(AppMessage::Settings(settings::Message::Toggle)),
        ),
    ]
    .spacing(SPACE_XS)
    .into()
}
