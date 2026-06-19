use explorer_core::ExplorerModel;
use iced::widget::{container, row, text, Space};
use iced::{alignment, Element, Fill, Theme};

use crate::fluent::{HEIGHT_STATUS_BAR, PAGE_PADDING_H, SPACE_SM};
use crate::message::Message;

pub struct StatusBarWidget;

impl StatusBarWidget {
    pub fn new() -> Self {
        Self
    }

    pub fn view(&self, model: &ExplorerModel) -> Element<'static, Message> {
        let path_text = model.current_path.display().to_string();

        container(
            row![
                text(model.status_text()).size(12).style(caption_text),
                Space::new().width(Fill),
                text(path_text).size(12).style(muted_caption),
            ]
            .align_y(alignment::Vertical::Center)
            .width(Fill),
        )
        .padding([SPACE_SM, PAGE_PADDING_H])
        .width(Fill)
        .height(HEIGHT_STATUS_BAR)
        .into()
    }
}

impl Default for StatusBarWidget {
    fn default() -> Self {
        Self::new()
    }
}

fn caption_text(theme: &Theme) -> iced::widget::text::Style {
    let palette = theme.extended_palette();
    iced::widget::text::Style {
        color: Some(palette.background.base.text),
    }
}

fn muted_caption(theme: &Theme) -> iced::widget::text::Style {
    let palette = theme.extended_palette();
    iced::widget::text::Style {
        color: Some(palette.background.base.text.scale_alpha(0.55)),
    }
}
