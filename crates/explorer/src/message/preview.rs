use explorer_app::{PreviewFile, TextEncoding};
use iced::widget::text_editor;

#[derive(Debug, Clone)]
pub enum Message {
    Close,
    Loaded(Result<PreviewFile, String>),
    OpenExternal,
    EncodingSelected(TextEncoding),
    TextScrolled(f32),
    TextIndexLoaded {
        id: u64,
        result: Result<Vec<u64>, String>,
    },
    TextWindowLoaded {
        id: u64,
        start: usize,
        result: Result<Vec<String>, String>,
    },
    DocumentEditor(text_editor::Action),
    ImageZoomIn,
    ImageZoomOut,
    ImageZoomReset,
    ImageWheelZoom(f32),
    HexScrolled(f32),
    HexSelect(usize),
    HexWindowLoaded {
        id: u64,
        start: usize,
        result: Result<Vec<u8>, String>,
    },
}
