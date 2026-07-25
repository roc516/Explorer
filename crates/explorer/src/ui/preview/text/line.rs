use iced::widget::text;
use iced::widget::text::Wrapping;
use iced::{Element, Font, Length};

use super::{Message, FONT_SIZE, LINE_HEIGHT};

pub fn view(content: String) -> Element<'static, Message> {
    text(content)
        .size(FONT_SIZE)
        .font(Font::MONOSPACE)
        .line_height(iced::widget::text::LineHeight::Absolute(iced::Pixels(
            LINE_HEIGHT,
        )))
        .wrapping(Wrapping::None)
        .width(Length::Shrink)
        .into()
}
