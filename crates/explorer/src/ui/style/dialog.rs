use iced::widget::rule;
use iced::Theme;

use crate::fluent::RADIUS_FLYOUT;

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

pub fn dialog_backdrop(theme: &Theme) -> iced::widget::container::Style {
    let palette = theme.extended_palette();
    let dim = if palette.is_dark {
        iced::Color::BLACK.scale_alpha(0.45)
    } else {
        iced::Color::BLACK.scale_alpha(0.32)
    };

    iced::widget::container::Style {
        background: Some(iced::Background::Color(dim)),
        ..Default::default()
    }
}
