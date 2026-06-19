use iced::widget::{button, container};
use iced::{alignment, Element, Length, Theme};
use lucide_icons::Icon;

use crate::fluent::NAV_BUTTON_SIZE;
use crate::widget::lucide_icon;

const NAV_ICON_SIZE: f32 = 16.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavIcon {
    Back,
    Forward,
    Up,
    Refresh,
    Settings,
}

impl NavIcon {
    fn lucide(self) -> Icon {
        match self {
            NavIcon::Back => Icon::ArrowLeft,
            NavIcon::Forward => Icon::ArrowRight,
            NavIcon::Up => Icon::ArrowUp,
            NavIcon::Refresh => Icon::RefreshCw,
            NavIcon::Settings => Icon::Settings,
        }
    }
}

pub fn nav_button_style(enabled: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let palette = theme.extended_palette();
        if !enabled {
            return button::Style {
                background: None,
                text_color: palette.background.base.text.scale_alpha(0.35),
                ..button::Style::default()
            };
        }

        match status {
            button::Status::Hovered => button::Style {
                background: Some(iced::Background::Color(
                    palette.background.strong.color.scale_alpha(0.45),
                )),
                text_color: palette.background.base.text,
                border: iced::Border {
                    radius: 4.0.into(),
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
                    radius: 4.0.into(),
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
}

pub fn nav_button<'a, Message: 'a + Clone>(
    kind: NavIcon,
    enabled: bool,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    let alpha = if enabled { 0.88 } else { 0.35 };
    let icon = container(lucide_icon::icon_muted::<Message>(kind.lucide(), NAV_ICON_SIZE, alpha))
        .width(Length::Fixed(NAV_ICON_SIZE))
        .height(Length::Fixed(NAV_ICON_SIZE))
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center);

    button(icon)
        .on_press_maybe(on_press)
        .width(Length::Fixed(NAV_BUTTON_SIZE))
        .height(Length::Fixed(NAV_BUTTON_SIZE))
        .padding(0)
        .style(nav_button_style(enabled))
        .into()
}
