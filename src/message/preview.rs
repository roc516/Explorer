use explorer_ui::{PreviewFile, TextEncoding};
use iced::widget::text_editor;

#[derive(Debug, Clone)]
pub enum Message {
    Close,
    PressInside,
    Loaded(Result<PreviewFile, String>),
    OpenExternal,
    EncodingSelected(TextEncoding),
    TextEditor(text_editor::Action),
    DocumentEditor(text_editor::Action),
    ImageZoomIn,
    ImageZoomOut,
    ImageZoomReset,
    ImageWheelZoom(f32),
}
