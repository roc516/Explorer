use iced::widget::pick_list;
use iced::Theme;

use crate::fluent::RADIUS_CONTROL;

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
