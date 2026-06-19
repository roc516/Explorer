use iced::widget::{button, pick_list, rule};
use iced::Theme;

use crate::fluent::{RADIUS_CONTROL, RADIUS_FLYOUT};

pub fn error_text(theme: &Theme) -> iced::widget::text::Style {
    let palette = theme.extended_palette();
    iced::widget::text::Style {
        color: Some(palette.danger.strong.color),
    }
}

pub fn dialog_container(theme: &Theme) -> iced::widget::container::Style {
    let palette = theme.extended_palette();
    iced::widget::container::Style {
        background: Some(iced::Background::Color(palette.background.base.color)),
        border: iced::Border {
            width: 1.0,
            color: palette.background.strong.color.scale_alpha(0.35),
            radius: RADIUS_FLYOUT.into(),
            ..Default::default()
        },
        shadow: iced::Shadow {
            color: iced::Color::BLACK.scale_alpha(0.16),
            offset: iced::Vector::new(0.0, 8.0),
            blur_radius: 24.0,
        },
        ..Default::default()
    }
}

pub fn dialog_divider(theme: &Theme) -> rule::Style {
    let palette = theme.extended_palette();
    rule::Style {
        color: palette.background.strong.color.scale_alpha(0.45),
        radius: 0.0.into(),
        fill_mode: rule::FillMode::Full,
        snap: false,
    }
}

pub fn pick_list_style(theme: &Theme, status: pick_list::Status) -> pick_list::Style {
    let palette = theme.extended_palette();
    let active = pick_list::Style {
        text_color: palette.background.base.text,
        background: palette.background.base.color.into(),
        placeholder_color: palette.background.base.text.scale_alpha(0.45),
        handle_color: palette.background.base.text.scale_alpha(0.72),
        border: iced::Border {
            radius: RADIUS_CONTROL.into(),
            width: 1.0,
            color: palette.background.strong.color.scale_alpha(0.55),
        },
    };

    match status {
        pick_list::Status::Active => active,
        pick_list::Status::Hovered | pick_list::Status::Opened { .. } => pick_list::Style {
            border: iced::Border {
                color: palette.primary.strong.color,
                ..active.border
            },
            ..active
        },
    }
}

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
