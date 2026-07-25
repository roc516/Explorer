use std::io::{Cursor, Read};

use office_oxide::{Document, DocumentFormat};

const MAX_BYTES: u64 = 32 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct WordPreview {
    pub content: String,
}

pub fn is_extension(ext: &str) -> bool {
    matches!(ext, "doc" | "docx")
}

pub fn load(reader: &mut dyn Read, size: u64, extension: &str) -> Result<WordPreview, String> {
    if size > MAX_BYTES {
        return Err("preview-too-large".to_string());
    }
    let format = DocumentFormat::from_extension(extension)
        .filter(|format| matches!(format, DocumentFormat::Doc | DocumentFormat::Docx))
        .ok_or_else(|| "preview-word-failed".to_string())?;
    let mut bytes = Vec::with_capacity(size as usize);
    reader
        .read_to_end(&mut bytes)
        .map_err(|err| err.to_string())?;
    let document = Document::from_reader(Cursor::new(bytes), format)
        .map_err(|_| "preview-word-failed".to_string())?;

    Ok(WordPreview {
        content: document.plain_text(),
    })
}
