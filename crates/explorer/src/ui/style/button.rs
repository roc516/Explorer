use iced::widget::button;
use iced::Theme;

use crate::fluent::RADIUS_CONTROL;

pub fn secondary_button(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();
    let base = button::Style {
        background: Some(iced::Background::Color(
            palette.background.strong.color.scale_alpha(0.28),
        )),
        text_color: palette.background.base.text,
        border: iced::Border {
            radius: RADIUS_CONTROL.into(),
            ..Default::default()
        },
        ..button::Style::default()
    };

    match status {
        button::Status::Hovered => button::Style {
            background: Some(iced::Background::Color(
                palette.background.strong.color.scale_alpha(0.42),
            )),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(iced::Background::Color(palette.primary.weak.color)),
            ..base
        },
        _ => base,
    }
}

pub fn icon_button(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();
    match status {
        button::Status::Hovered => button::Style {
            background: Some(iced::Background::Color(
                palette.background.strong.color.scale_alpha(0.4),
            )),
            text_color: palette.background.base.text,
            border: iced::Border {
                radius: RADIUS_CONTROL.into(),
                ..Default::default()
            },
            ..button::Style::default()
        },
        button::Status::Pressed => button::Style {
            background: Some(iced::Background::Color(
                palette.primary.weak.color.scale_alpha(0.85),
            )),
            text_color: palette.background.base.text,
            border: iced::Border {
                radius: RADIUS_CONTROL.into(),
                ..Default::default()
            },
            ..button::Style::default()
        },
        _ => button::Style {
            background: None,
            text_color: palette.background.base.text,
            ..button::Style::default()
        },
    }
}
