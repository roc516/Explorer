use iced::widget::text;
use iced::{Element, Font, Theme};
use lucide_icons::Icon;

/// Stateless Lucide icon control — renders a glyph from the Lucide icon font.
#[derive(Debug, Clone, Copy)]
pub struct LucideIcon {
    icon: Icon,
    size: f32,
    alpha: Option<f32>,
}

impl LucideIcon {
    pub fn new(icon: Icon) -> Self {
        Self {
            icon,
            size: 16.0,
            alpha: None,
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Tint with theme text color at the given alpha (0.0–1.0).
    pub fn muted(mut self, alpha: f32) -> Self {
        self.alpha = Some(alpha);
        self
    }

    pub fn view<'a, Message: 'a>(self) -> Element<'a, Message> {
        let alpha = self.alpha;
        text(char::from(self.icon).to_string())
            .font(Font::with_name("lucide"))
            .size(self.size)
            .style(move |theme: &Theme| iced::widget::text::Style {
                color: Some(match alpha {
                    Some(alpha) => theme
                        .extended_palette()
                        .background
                        .base
                        .text
                        .scale_alpha(alpha),
                    None => theme.extended_palette().background.base.text,
                }),
            })
            .into()
    }
}

impl<'a, Message: 'a> From<LucideIcon> for Element<'a, Message> {
    fn from(value: LucideIcon) -> Self {
        value.view()
    }
}
