use iced::widget::text;
use iced::{Element, Fill, Font};

use crate::message::preview;

use super::{FONT_SIZE, LINE_HEIGHT};

pub fn view(content: String) -> Element<'static, preview::Message> {
    text(content)
        .size(FONT_SIZE)
        .font(Font::MONOSPACE)
        .line_height(iced::widget::text::LineHeight::Absolute(iced::Pixels(
            LINE_HEIGHT,
        )))
        .width(Fill)
        .into()
}
