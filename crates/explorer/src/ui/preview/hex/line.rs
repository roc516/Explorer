use iced::widget::{container, mouse_area, row, text, Space};
use iced::{alignment, Element, Fill, Font, Length, Theme};

use crate::fluent::{SPACE_MD, SPACE_SM};
use super::Message;
use crate::ui::preview::status_muted_text;

use super::byte;
use super::{
    ascii_char, ASCII_WIDTH, BYTES_PER_LINE, FONT_SIZE, LINE_HEIGHT, OFFSET_WIDTH,
};

pub fn view(
    offset: usize,
    chunk: &[u8],
    file_size: usize,
    selected: Option<usize>,
) -> Element<'static, Message> {
    let mut hex_cells: Vec<Element<'static, Message>> =
        Vec::with_capacity(BYTES_PER_LINE + 1);
    for i in 0..BYTES_PER_LINE {
        if i == 8 {
            hex_cells.push(Space::new().width(Length::Fixed(SPACE_SM)).into());
        }
        let index = offset + i;
        if index < file_size {
            if let Some(value) = chunk.get(i).copied() {
                hex_cells.push(byte::view(index, value, selected == Some(index)));
            } else {
                hex_cells.push(byte::placeholder());
            }
        } else {
            hex_cells.push(byte::placeholder());
        }
    }

    let ascii: String = chunk.iter().copied().map(ascii_char).collect();
    let ascii_row = mouse_area(
        container(
            text(ascii)
                .size(FONT_SIZE)
                .font(Font::MONOSPACE)
                .style(ascii_text),
        )
        .width(Length::Fixed(ASCII_WIDTH))
        .height(Length::Fixed(LINE_HEIGHT))
        .align_y(alignment::Vertical::Center),
    )
    .on_press(Message::Select(offset));

    row![
        container(
            text(format!("{offset:08X}"))
                .size(FONT_SIZE)
                .font(Font::MONOSPACE)
                .style(status_muted_text),
        )
        .width(Length::Fixed(OFFSET_WIDTH))
        .height(Length::Fixed(LINE_HEIGHT))
        .align_y(alignment::Vertical::Center),
        row(hex_cells).align_y(alignment::Vertical::Center),
        Space::new().width(Length::Fixed(SPACE_MD)),
        ascii_row,
    ]
    .spacing(SPACE_SM)
    .align_y(alignment::Vertical::Center)
    .width(Fill)
    .height(Length::Fixed(LINE_HEIGHT))
    .into()
}

fn ascii_text(theme: &Theme) -> iced::widget::text::Style {
    let palette = theme.extended_palette();
    iced::widget::text::Style {
        color: Some(palette.background.base.text),
    }
}
