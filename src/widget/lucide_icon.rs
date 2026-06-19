use iced::widget::text;
use iced::{Element, Font, Theme};
use lucide_icons::Icon;

pub fn icon<'a, Message: 'a>(icon: Icon, size: f32) -> Element<'a, Message> {
    icon_colored(icon, size, |theme| theme.extended_palette().background.base.text)
}

pub fn icon_muted<'a, Message: 'a>(icon: Icon, size: f32, alpha: f32) -> Element<'a, Message> {
    icon_colored(icon, size, move |theme| {
        theme
            .extended_palette()
            .background
            .base
            .text
            .scale_alpha(alpha)
    })
}

fn icon_colored<'a, Message: 'a>(
    icon: Icon,
    size: f32,
    color: impl Fn(&Theme) -> iced::Color + 'a,
) -> Element<'a, Message> {
    text(char::from(icon).to_string())
        .font(Font::with_name("lucide"))
        .size(size)
        .style(move |theme| iced::widget::text::Style {
            color: Some(color(theme)),
        })
        .into()
}
