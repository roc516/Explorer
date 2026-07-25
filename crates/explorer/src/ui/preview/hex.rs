mod byte;
mod header;
mod line;
mod status_bar;

use explorer_app::{ids, HexPreview, LanguageBundle, PreviewFile, PreviewKind};
use iced::widget::{column, scrollable, Space};
use iced::widget::scrollable::Direction;
use iced::{Element, Fill, Length, Task};

use crate::fluent::{
    HEIGHT_PREVIEW_BODY, HEIGHT_PREVIEW_STATUS_BAR, SPACE_LG, SPACE_XS,
};

use super::preview_message;

pub use status_bar::view as status_bar;

pub(crate) const BYTES_PER_LINE: usize = 16;
pub(crate) const LINE_HEIGHT: f32 = 22.0;
pub(crate) const OFFSET_WIDTH: f32 = 72.0;
pub(crate) const ASCII_WIDTH: f32 = 140.0;
pub(crate) const BYTE_WIDTH: f32 = 24.0;
pub(crate) const FONT_SIZE: f32 = 12.0;
const OVERSCAN_LINES: usize = 4;
/// Extra lines kept beyond the visible range so small scrolls reuse the cache.
const WINDOW_MARGIN_LINES: usize = 16;

#[derive(Debug, Clone)]
pub enum Message {
    Scrolled(f32),
    Select(usize),
    WindowLoaded {
        id: u64,
        start: usize,
        result: Result<Vec<u8>, String>,
    },
}


#[derive(Debug, Clone)]
struct ByteWindow {
    start: usize,
    data: Vec<u8>,
}

impl ByteWindow {
    fn end(&self) -> usize {
        self.start + self.data.len()
    }

    fn covers(&self, start: usize, end: usize) -> bool {
        start >= self.start && end <= self.end()
    }

    fn slice(&self, offset: usize, len: usize) -> Option<&[u8]> {
        let rel = offset.checked_sub(self.start)?;
        if rel >= self.data.len() {
            return Some(&[]);
        }
        let end = (rel + len).min(self.data.len());
        Some(&self.data[rel..end])
    }

    fn get(&self, offset: usize) -> Option<u8> {
        offset
            .checked_sub(self.start)
            .and_then(|rel| self.data.get(rel).copied())
    }
}

#[derive(Debug, Clone)]
struct PendingWindow {
    start: usize,
    end: usize,
}

impl PendingWindow {
    fn covers(&self, start: usize, end: usize) -> bool {
        start >= self.start && end <= self.end
    }
}

#[derive(Debug, Clone)]
pub struct Hex {
    pub scroll_y: f32,
    pub selected: Option<usize>,
    window: Option<ByteWindow>,
    pending: Option<PendingWindow>,
    load_id: u64,
    load_error: Option<String>,
}

impl Hex {
    pub fn for_file(file: &PreviewFile) -> Option<(Self, Task<Message>)> {
        let PreviewKind::Hex(preview) = &file.kind else {
            return None;
        };
        let mut state = Self::new();
        let task = state.request_window(preview);
        Some((state, task))
    }

    pub fn new() -> Self {
        Self {
            scroll_y: 0.0,
            selected: None,
            window: None,
            pending: None,
            load_id: 0,
            load_error: None,
        }
    }

    pub fn on_scroll(&mut self, preview: &HexPreview, y: f32) -> Task<Message> {
        self.scroll_y = y;
        self.request_window(preview)
    }

    pub fn select(&mut self, index: usize) {
        self.selected = Some(index);
    }

    pub fn apply_window(
        &mut self,
        id: u64,
        start: usize,
        result: Result<Vec<u8>, String>,
    ) {
        if id != self.load_id {
            return;
        }
        self.pending = None;
        match result {
            Ok(data) => {
                self.window = Some(ByteWindow { start, data });
                self.load_error = None;
            }
            Err(message) => {
                self.load_error = Some(message);
            }
        }
    }

    fn request_window(&mut self, preview: &HexPreview) -> Task<Message> {
        if preview.size == 0 {
            self.window = Some(ByteWindow {
                start: 0,
                data: Vec::new(),
            });
            self.pending = None;
            self.load_error = None;
            return Task::none();
        }

        let line_count = preview.size.div_ceil(BYTES_PER_LINE as u64) as usize;
        let (first, last) = visible_line_range(self.scroll_y, line_count);
        let need_start = first * BYTES_PER_LINE;
        let need_end = (last * BYTES_PER_LINE).min(preview.size as usize);

        if let Some(window) = &self.window {
            if window.covers(need_start, need_end) {
                return Task::none();
            }
        }

        if let Some(pending) = &self.pending {
            if pending.covers(need_start, need_end) {
                return Task::none();
            }
        }

        let load_first = first.saturating_sub(WINDOW_MARGIN_LINES);
        let load_last = (last + WINDOW_MARGIN_LINES).min(line_count);
        let start = load_first * BYTES_PER_LINE;
        let end = (load_last * BYTES_PER_LINE).min(preview.size as usize);
        let len = end.saturating_sub(start);

        self.load_id = self.load_id.wrapping_add(1);
        let id = self.load_id;
        self.pending = Some(PendingWindow { start, end });
        self.load_error = None;

        let preview = preview.clone();
        Task::perform(
            async move { (id, start, preview.read_range(start as u64, len)) },
            |(id, start, result)| Message::WindowLoaded { id, start, result },
        )
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
) -> Element<'static, Message> {
    if preview.size == 0 {
        return preview_message(bundle.tr(ids::PREVIEW_HEX_EMPTY), false);
    }

    if let Some(error) = &state.load_error {
        return preview_message(error.clone(), true);
    }

    let Some(window) = &state.window else {
        return crate::ui::loading::view_tr(bundle);
    };

    let line_count = preview.size.div_ceil(BYTES_PER_LINE as u64) as usize;
    let (first, last) = visible_line_range(state.scroll_y, line_count);

    let mut lines: Vec<Element<'static, Message>> = Vec::with_capacity(last - first + 2);
    if first > 0 {
        lines.push(
            Space::new()
                .height(Length::Fixed(first as f32 * LINE_HEIGHT))
                .into(),
        );
    }

    for row in first..last {
        let offset = row * BYTES_PER_LINE;
        let chunk = window
            .slice(offset, BYTES_PER_LINE)
            .unwrap_or(&[])
            .to_vec();
        lines.push(line::view(offset, &chunk, preview.size as usize, state.selected));
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
    .on_scroll(|viewport| Message::Scrolled(viewport.absolute_offset().y))
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

pub(crate) fn selected_byte(state: &Hex) -> Option<u8> {
    let index = state.selected?;
    state.window.as_ref()?.get(index)
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
