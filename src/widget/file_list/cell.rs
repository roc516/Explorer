use iced::widget::{container, text, Space};
use iced::widget::text::Wrapping;
use iced::{Element, Length};

use crate::fluent::SPACE_SM;

const ELLIPSIS: char = '…';

pub(crate) fn column_gap<Message: 'static>() -> Element<'static, Message> {
    Space::new()
        .width(Length::Fixed(SPACE_SM))
        .into()
}

pub(crate) fn clipped_cell<Message: 'static>(
    label: impl AsRef<str>,
    width: f32,
    size: f32,
) -> Element<'static, Message> {
    let display = truncate_to_width(label.as_ref(), width, size);
    container(
        text(display)
            .size(size)
            .wrapping(Wrapping::None),
    )
    .width(Length::Fixed(width))
    .clip(true)
    .into()
}

fn truncate_to_width(text: &str, width: f32, font_size: f32) -> String {
    if estimate_text_width(text, font_size) <= width {
        return text.to_string();
    }

    let ellipsis_width = char_width_px(ELLIPSIS, font_size);
    let budget = (width - ellipsis_width).max(0.0);
    let mut used = 0.0;
    let mut result = String::new();

    for ch in text.chars() {
        let char_width = char_width_px(ch, font_size);
        if used + char_width > budget {
            break;
        }
        used += char_width;
        result.push(ch);
    }

    result.push(ELLIPSIS);
    result
}

fn estimate_text_width(text: &str, font_size: f32) -> f32 {
    text.chars()
        .map(|ch| char_width_px(ch, font_size))
        .sum()
}

fn char_width_px(ch: char, font_size: f32) -> f32 {
    if ch.is_ascii() {
        font_size * 0.52
    } else {
        font_size
    }
}
