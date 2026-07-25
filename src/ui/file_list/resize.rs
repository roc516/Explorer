use iced::event::Status;
use iced::mouse::{self, Interaction};
use iced::widget::{container, mouse_area, Space};
use iced::{alignment, Element, Event, Length, Theme};

use crate::fluent::{HEIGHT_LIST_ROW, SPACE_SM, SPACE_XS};

use super::columns::Column;
use super::message::Message;

const DIVIDER_LINE_HEIGHT: f32 = HEIGHT_LIST_ROW - SPACE_XS * 2.0;

pub(crate) fn column_resize_listener(
    event: Event,
    _status: Status,
    _window: iced::window::Id,
) -> Option<Message> {
    match event {
        Event::Mouse(mouse::Event::CursorMoved { position }) => {
            Some(Message::ColumnResizeMoved(position.x))
        }
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
            Some(Message::ColumnResizeEnded)
        }
        _ => None,
    }
}

pub(crate) fn column_divider(column: Column, active: bool) -> Element<'static, Message> {
    mouse_area(
        container(divider_line(active))
            .width(Length::Fixed(SPACE_SM))
            .height(Length::Fixed(HEIGHT_LIST_ROW))
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center),
    )
    .on_press(Message::ColumnResizeStarted(column))
    .interaction(Interaction::ResizingColumn)
    .into()
}

fn divider_line(active: bool) -> Element<'static, Message> {
    container(Space::new())
        .width(Length::Fixed(1.0))
        .height(Length::Fixed(DIVIDER_LINE_HEIGHT))
        .style(move |theme| divider_line_style(theme, active))
        .into()
}

fn divider_line_style(theme: &Theme, active: bool) -> iced::widget::container::Style {
    let palette = theme.extended_palette();
    let color = if active {
        palette.primary.strong.color
    } else {
        palette.background.strong.color.scale_alpha(0.45)
    };

    iced::widget::container::Style {
        background: Some(iced::Background::Color(color)),
        ..Default::default()
    }
}
