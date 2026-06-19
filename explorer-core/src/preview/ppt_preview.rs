use std::io::{Cursor, Read};

use office_oxide::{Document, DocumentFormat};

use super::io;

const MAX_BYTES: u64 = 32 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct PptPreview {
    pub content: String,
    pub slide_count: usize,
}

pub fn is_extension(ext: &str) -> bool {
    matches!(ext, "ppt" | "pptx")
}

pub fn load(reader: &mut dyn Read, size: u64, extension: &str) -> Result<PptPreview, String> {
    let format = DocumentFormat::from_extension(extension)
        .filter(|format| matches!(format, DocumentFormat::Ppt | DocumentFormat::Pptx))
        .ok_or_else(|| "preview-ppt-failed".to_string())?;
    let document = Document::from_reader(
        Cursor::new(io::copy_limited(reader, MAX_BYTES, Some(size))?),
        format,
    )
    .map_err(|_| "preview-ppt-failed".to_string())?;
    let slide_count = document
        .as_pptx()
        .map(|ppt| ppt.slides.len())
        .or_else(|| document.as_ppt().map(|ppt| ppt.slides.len()))
        .unwrap_or(0);

    Ok(PptPreview {
        content: document.plain_text(),
        slide_count,
    })
}
