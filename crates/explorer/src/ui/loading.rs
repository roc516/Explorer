use explorer_app::{ids, LanguageBundle};
use iced::widget::{column, container, text};
use iced::{alignment, Element, Fill, Theme};
use lucide_icons::Icon;

use crate::fluent::SPACE_SM;
use crate::widget::LucideIcon;

const ICON_SIZE: f32 = 20.0;
const LABEL_SIZE: f32 = 14.0;

/// Centered loading panel used across list, preview, and other panes.
pub fn view<'a, Message: 'a>(label: impl Into<String>) -> Element<'a, Message> {
    let label = label.into();
    container(
        column![
            LucideIcon::new(Icon::Loader2).size(ICON_SIZE).muted(0.55),
            text(label).size(LABEL_SIZE).style(loading_text),
        ]
        .spacing(SPACE_SM)
        .align_x(alignment::Horizontal::Center),
    )
    .width(Fill)
    .height(Fill)
    .align_x(alignment::Horizontal::Center)
    .align_y(alignment::Vertical::Center)
    .into()
}

/// Loading panel with the shared localized “Loading…” string.
pub fn view_tr<'a, Message: 'a>(bundle: LanguageBundle) -> Element<'a, Message> {
    view(bundle.tr(ids::STATUS_LOADING))
}

fn loading_text(theme: &Theme) -> iced::widget::text::Style {
    let palette = theme.extended_palette();
    iced::widget::text::Style {
        color: Some(palette.background.base.text.scale_alpha(0.55)),
    }
}
