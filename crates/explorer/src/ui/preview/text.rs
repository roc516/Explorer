mod line;
mod status_bar;

use explorer_app::{
    ids, needs_reindex, LanguageBundle, PreviewFile, PreviewKind, TextEncoding, TextPreview,
};
use iced::widget::{column, scrollable, Space};
use iced::widget::scrollable::Direction;
use iced::{Element, Fill, Length, Task};

use crate::fluent::{HEIGHT_PREVIEW_BODY, HEIGHT_PREVIEW_STATUS_BAR, SPACE_LG};

use super::preview_message;

pub use status_bar::view as status_bar;

pub(crate) const LINE_HEIGHT: f32 = 20.0;
pub(crate) const FONT_SIZE: f32 = 13.0;
const OVERSCAN_LINES: usize = 4;
const WINDOW_MARGIN_LINES: usize = 32;

#[derive(Debug, Clone)]
pub enum Message {
    Scrolled(f32),
    IndexLoaded {
        id: u64,
        result: Result<Vec<u64>, String>,
    },
    WindowLoaded {
        id: u64,
        start: usize,
        result: Result<Vec<String>, String>,
    },
    EncodingSelected(TextEncoding),
}


#[derive(Debug, Clone)]
struct LineWindow {
    start: usize,
    lines: Vec<String>,
}

impl LineWindow {
    fn end(&self) -> usize {
        self.start + self.lines.len()
    }

    fn covers(&self, start: usize, end: usize) -> bool {
        start >= self.start && end <= self.end()
    }

    fn get(&self, line: usize) -> Option<&str> {
        line.checked_sub(self.start)
            .and_then(|rel| self.lines.get(rel).map(String::as_str))
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
pub struct Text {
    pub encoding: TextEncoding,
    pub encoding_error: Option<String>,
    scroll_y: f32,
    line_offsets: Option<Vec<u64>>,
    window: Option<LineWindow>,
    pending: Option<PendingWindow>,
    load_id: u64,
    load_error: Option<String>,
    indexing: bool,
}

impl Text {
    pub fn for_file(file: &PreviewFile) -> Option<(Self, Task<Message>)> {
        let PreviewKind::Text(preview) = &file.kind else {
            return None;
        };
        let mut state = Self::new();
        let task = state.start_index(preview);
        Some((state, task))
    }

    pub fn new() -> Self {
        Self {
            encoding: TextEncoding::Auto,
            encoding_error: None,
            scroll_y: 0.0,
            line_offsets: None,
            window: None,
            pending: None,
            load_id: 0,
            load_error: None,
            indexing: false,
        }
    }

    pub fn on_scroll(&mut self, preview: &TextPreview, y: f32) -> Task<Message> {
        self.scroll_y = y;
        self.request_window(preview)
    }

    pub fn select_encoding(
        &mut self,
        preview: &TextPreview,
        encoding: TextEncoding,
    ) -> Task<Message> {
        let previous = self.encoding;
        self.encoding = encoding;
        self.encoding_error = None;

        if needs_reindex(
            preview.resolve_encoding(previous),
            preview.resolve_encoding(encoding),
        ) || self.line_offsets.is_none()
        {
            self.line_offsets = None;
            self.window = None;
            self.pending = None;
            return self.start_index(preview);
        }

        self.window = None;
        self.pending = None;
        self.request_window(preview)
    }

    pub fn apply_index(
        &mut self,
        preview: &TextPreview,
        id: u64,
        result: Result<Vec<u64>, String>,
    ) -> Task<Message> {
        if id != self.load_id {
            return Task::none();
        }
        self.indexing = false;
        match result {
            Ok(offsets) => {
                self.line_offsets = Some(offsets);
                self.load_error = None;
                self.request_window(preview)
            }
            Err(message) => {
                self.load_error = Some(message);
                Task::none()
            }
        }
    }

    pub fn apply_window(
        &mut self,
        id: u64,
        start: usize,
        result: Result<Vec<String>, String>,
    ) {
        if id != self.load_id {
            return;
        }
        self.pending = None;
        match result {
            Ok(lines) => {
                self.window = Some(LineWindow { start, lines });
                self.load_error = None;
                self.encoding_error = None;
            }
            Err(message) => {
                if message == "preview-decode-failed" {
                    self.encoding_error = Some(message);
                } else {
                    self.load_error = Some(message);
                }
            }
        }
    }

    fn start_index(&mut self, preview: &TextPreview) -> Task<Message> {
        self.indexing = true;
        self.load_error = None;
        self.load_id = self.load_id.wrapping_add(1);
        let id = self.load_id;
        let encoding = self.encoding;
        let preview = preview.clone();
        Task::perform(
            async move { (id, preview.build_line_index(encoding)) },
            |(id, result)| Message::IndexLoaded { id, result },
        )
    }

    fn request_window(&mut self, preview: &TextPreview) -> Task<Message> {
        let Some(offsets) = self.line_offsets.as_ref() else {
            return Task::none();
        };
        let line_count = offsets.len();
        if line_count == 0 {
            self.window = Some(LineWindow {
                start: 0,
                lines: Vec::new(),
            });
            return Task::none();
        }

        let (first, last) = visible_line_range(self.scroll_y, line_count);
        if let Some(window) = &self.window {
            if window.covers(first, last) {
                return Task::none();
            }
        }
        if let Some(pending) = &self.pending {
            if pending.covers(first, last) {
                return Task::none();
            }
        }

        let load_first = first.saturating_sub(WINDOW_MARGIN_LINES);
        let load_last = (last + WINDOW_MARGIN_LINES).min(line_count);

        self.load_id = self.load_id.wrapping_add(1);
        let id = self.load_id;
        self.pending = Some(PendingWindow {
            start: load_first,
            end: load_last,
        });

        let offsets = offsets.clone();
        let encoding = self.encoding;
        let preview = preview.clone();
        Task::perform(
            async move {
                (
                    id,
                    load_first,
                    preview.read_lines(&offsets, load_first, load_last, encoding),
                )
            },
            |(id, start, result)| Message::WindowLoaded { id, start, result },
        )
    }
}

impl Default for Text {
    fn default() -> Self {
        Self::new()
    }
}

pub fn view(
    bundle: LanguageBundle,
    preview: &TextPreview,
    state: &Text,
) -> Element<'static, Message> {
    if let Some(error) = &state.load_error {
        let message = if error == "preview-decode-failed" {
            bundle.tr(ids::PREVIEW_DECODE_FAILED)
        } else {
            error.clone()
        };
        return preview_message(message, true);
    }

    if state.indexing || state.line_offsets.is_none() {
        return crate::ui::loading::view_tr(bundle);
    }

    let Some(offsets) = &state.line_offsets else {
        return crate::ui::loading::view_tr(bundle);
    };

    if preview.size == 0 {
        return preview_message(String::new(), false);
    }

    let Some(window) = &state.window else {
        return crate::ui::loading::view_tr(bundle);
    };

    let line_count = offsets.len();
    let (first, last) = visible_line_range(state.scroll_y, line_count);

    let mut rows: Vec<Element<'static, Message>> = Vec::with_capacity(last - first + 2);
    if first > 0 {
        rows.push(
            Space::new()
                .height(Length::Fixed(first as f32 * LINE_HEIGHT))
                .into(),
        );
    }

    for row in first..last {
        let content = window.get(row).unwrap_or("").to_string();
        rows.push(line::view(content));
    }

    let trailing = line_count.saturating_sub(last);
    if trailing > 0 {
        rows.push(
            Space::new()
                .height(Length::Fixed(trailing as f32 * LINE_HEIGHT))
                .into(),
        );
    }

    scrollable(
        column(rows)
            .width(Length::Shrink)
            .height(Length::Fixed(line_count as f32 * LINE_HEIGHT)),
    )
    .direction(Direction::Both {
        vertical: scrollable::Scrollbar::default(),
        horizontal: scrollable::Scrollbar::default(),
    })
    .on_scroll(|viewport| Message::Scrolled(viewport.absolute_offset().y))
    .width(Fill)
    .height(Fill)
    .into()
}

fn body_viewport_height() -> f32 {
    HEIGHT_PREVIEW_BODY - HEIGHT_PREVIEW_STATUS_BAR - 1.0 - 2.0 * SPACE_LG
}

fn visible_line_range(scroll_y: f32, line_count: usize) -> (usize, usize) {
    let viewport_h = body_viewport_height();
    let first = ((scroll_y / LINE_HEIGHT).floor() as usize).saturating_sub(OVERSCAN_LINES);
    let visible = ((viewport_h / LINE_HEIGHT).ceil() as usize) + OVERSCAN_LINES * 2;
    let last = (first + visible).min(line_count);
    (first, last)
}
