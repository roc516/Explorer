mod byte;
mod header;
mod line;
mod status_bar;

use explorer_app::{ids, HexPreview, LanguageBundle, PreviewFile, PreviewKind};
use iced::widget::{column, scrollable, Space};
use iced::widget::scrollable::Direction;
use iced::{Element, Fill, Length};

use crate::fluent::{
    HEIGHT_PREVIEW_BODY, HEIGHT_PREVIEW_STATUS_BAR, SPACE_LG, SPACE_XS,
};
use crate::message::preview;

use super::preview_message;

pub use status_bar::view as status_bar;

pub(crate) const BYTES_PER_LINE: usize = 16;
pub(crate) const LINE_HEIGHT: f32 = 22.0;
pub(crate) const OFFSET_WIDTH: f32 = 72.0;
pub(crate) const ASCII_WIDTH: f32 = 140.0;
pub(crate) const BYTE_WIDTH: f32 = 24.0;
pub(crate) const FONT_SIZE: f32 = 12.0;
const OVERSCAN_LINES: usize = 4;

#[derive(Debug, Clone)]
pub struct Hex {
    pub scroll_y: f32,
    pub selected: Option<usize>,
}

impl Hex {
    pub fn for_file(file: &PreviewFile) -> Option<Self> {
        if !matches!(file.kind, PreviewKind::Hex(_)) {
            return None;
        }
        Some(Self::new())
    }

    pub fn new() -> Self {
        Self {
            scroll_y: 0.0,
            selected: None,
        }
    }

    pub fn on_scroll(&mut self, y: f32) {
        self.scroll_y = y;
    }

    pub fn select(&mut self, index: usize) {
        self.selected = Some(index);
    }
}

impl Default for Hex {
    fn default() -> Self {
        Self::new()
    }
}

pub fn view(
    bundle: LanguageBundle,
    preview: &HexPreview,
    state: &Hex,
) -> Element<'static, preview::Message> {
    if preview.bytes.is_empty() {
        return preview_message(bundle.tr(ids::PREVIEW_HEX_EMPTY), false);
    }

    let line_count = preview.bytes.len().div_ceil(BYTES_PER_LINE);
    let (first, last) = visible_line_range(state.scroll_y, line_count);

    let mut lines: Vec<Element<'static, preview::Message>> = Vec::with_capacity(last - first + 2);
    if first > 0 {
        lines.push(
            Space::new()
                .height(Length::Fixed(first as f32 * LINE_HEIGHT))
                .into(),
        );
    }

    for row in first..last {
        lines.push(line::view(preview, row * BYTES_PER_LINE, state.selected));
    }

    let trailing = line_count.saturating_sub(last);
    if trailing > 0 {
        lines.push(
            Space::new()
                .height(Length::Fixed(trailing as f32 * LINE_HEIGHT))
                .into(),
        );
    }

    let body = scrollable(
        column(lines)
            .width(Fill)
            .height(Length::Fixed(line_count as f32 * LINE_HEIGHT)),
    )
    .direction(Direction::Vertical(scrollable::Scrollbar::default()))
    .on_scroll(|viewport| preview::Message::HexScrolled(viewport.absolute_offset().y))
    .width(Fill)
    .height(Fill);

    column![header::view(), body]
        .spacing(SPACE_XS)
        .width(Fill)
        .height(Fill)
        .into()
}

pub(crate) fn ascii_char(byte: u8) -> char {
    match byte {
        b' '..=b'~' => byte as char,
        _ => '.',
    }
}

fn body_viewport_height() -> f32 {
    HEIGHT_PREVIEW_BODY - HEIGHT_PREVIEW_STATUS_BAR - 1.0 - 2.0 * SPACE_LG - LINE_HEIGHT - SPACE_XS
}

fn visible_line_range(scroll_y: f32, line_count: usize) -> (usize, usize) {
    let viewport_h = body_viewport_height();
    let first = ((scroll_y / LINE_HEIGHT).floor() as usize).saturating_sub(OVERSCAN_LINES);
    let visible = ((viewport_h / LINE_HEIGHT).ceil() as usize) + OVERSCAN_LINES * 2;
    let last = (first + visible).min(line_count);
    (first, last)
}
