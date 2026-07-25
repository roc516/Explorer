use explorer_app::{PreviewFile, TextEncoding};
use iced::widget::text_editor;

#[derive(Debug, Clone)]
pub enum Message {
    Close,
    Loaded(Result<PreviewFile, String>),
    OpenExternal,
    EncodingSelected(TextEncoding),
    TextEditor(text_editor::Action),
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
