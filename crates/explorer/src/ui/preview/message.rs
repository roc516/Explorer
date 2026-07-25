use explorer_app::PreviewFile;

use super::{document, hex, image, text};

#[derive(Debug, Clone)]
pub enum Message {
    Close,
    Loaded(Result<PreviewFile, String>),
    OpenExternal,
    Text(text::Message),
    Hex(hex::Message),
    Image(image::Message),
    Document(document::Message),
}
