use iced::widget::{button, container, text};
use iced::{alignment, Element, Font, Length, Theme};

use crate::message::preview;

use super::{BYTE_WIDTH, FONT_SIZE, LINE_HEIGHT};

pub fn view(index: usize, byte: u8, selected: bool) -> Element<'static, preview::Message> {
    button(
        container(
            text(format!("{byte:02X}"))
                .size(FONT_SIZE)
                .font(Font::MONOSPACE)
                .style(if selected { selected_text } else { text_style }),
        )
        .width(Length::Fixed(BYTE_WIDTH))
        .height(Length::Fixed(LINE_HEIGHT))
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center),
    )
    .on_press(preview::Message::HexSelect(index))
    .padding(0)
    .style(if selected { selected_button } else { button_style })
    .into()
}

pub fn placeholder() -> Element<'static, preview::Message> {
    container(text("  ").size(FONT_SIZE).font(Font::MONOSPACE))
        .width(Length::Fixed(BYTE_WIDTH))
        .height(Length::Fixed(LINE_HEIGHT))
        .into()
}

fn text_style(theme: &Theme) -> iced::widget::text::Style {
    let palette = theme.extended_palette();
    iced::widget::text::Style {
        color: Some(palette.background.base.text),
    }
}

fn selected_text(theme: &Theme) -> iced::widget::text::Style {
    let palette = theme.extended_palette();
    iced::widget::text::Style {
        color: Some(palette.primary.base.text),
    }
}

fn button_style(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();
    match status {
        button::Status::Hovered => button::Style {
            background: Some(iced::Background::Color(
                palette.background.strong.color.scale_alpha(0.35),
            )),
            text_color: palette.background.base.text,
            border: iced::Border::default(),
            ..button::Style::default()
        },
        _ => button::Style {
            background: None,
            text_color: palette.background.base.text,
            border: iced::Border::default(),
            ..button::Style::default()
        },
    }
}

fn selected_button(theme: &Theme, _status: button::Status) -> button::Style {
    let palette = theme.extended_palette();
    button::Style {
        background: Some(iced::Background::Color(palette.primary.strong.color)),
        text_color: palette.primary.base.text,
        border: iced::Border {
            radius: 2.0.into(),
            ..Default::default()
        },
        ..button::Style::default()
    }
}
