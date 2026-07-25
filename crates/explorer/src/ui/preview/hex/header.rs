use iced::widget::{container, row, text, Space};
use iced::{alignment, Element, Fill, Font, Length};

use crate::fluent::SPACE_SM;
use super::Message;
use crate::ui::preview::status_muted_text;

use super::{BYTE_WIDTH, BYTES_PER_LINE, FONT_SIZE, OFFSET_WIDTH, ASCII_WIDTH};
use crate::fluent::SPACE_MD;

pub fn view() -> Element<'static, Message> {
    let mut hex_headers: Vec<Element<'static, Message>> =
        Vec::with_capacity(BYTES_PER_LINE + 1);
    for i in 0..BYTES_PER_LINE {
        if i == 8 {
            hex_headers.push(Space::new().width(Length::Fixed(SPACE_SM)).into());
        }
        hex_headers.push(
            container(
                text(format!("{i:02X}"))
                    .size(FONT_SIZE)
                    .font(Font::MONOSPACE)
                    .style(status_muted_text),
            )
            .width(Length::Fixed(BYTE_WIDTH))
            .align_x(alignment::Horizontal::Center)
            .into(),
        );
    }

    row![
        container(
            text("Offset")
                .size(FONT_SIZE)
                .font(Font::MONOSPACE)
                .style(status_muted_text),
        )
        .width(Length::Fixed(OFFSET_WIDTH))
        .align_y(alignment::Vertical::Center),
        row(hex_headers).align_y(alignment::Vertical::Center),
        Space::new().width(Length::Fixed(SPACE_MD)),
        container(
            text("ASCII")
                .size(FONT_SIZE)
                .font(Font::MONOSPACE)
                .style(status_muted_text),
        )
        .width(Length::Fixed(ASCII_WIDTH))
        .align_y(alignment::Vertical::Center),
    ]
    .spacing(SPACE_SM)
    .align_y(alignment::Vertical::Center)
    .width(Fill)
    .into()
}
