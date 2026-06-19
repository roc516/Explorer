use std::path::Path;

use office_oxide::Document;

const MAX_BYTES: u64 = 32 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct WordPreview {
    pub content: String,
}

impl WordPreview {
    pub fn from_path(path: &Path) -> Result<Self, String> {
        let document = Document::open(path).map_err(|_| "preview-word-failed".to_string())?;
        Ok(Self {
            content: document.plain_text(),
        })
    }
}

pub fn is_extension(ext: &str) -> bool {
    matches!(ext, "doc" | "docx")
}

pub fn load_from_path(path: &Path, size: u64) -> Result<WordPreview, String> {
    if size > MAX_BYTES {
        return Err("preview-too-large".to_string());
    }

    WordPreview::from_path(path)
}
