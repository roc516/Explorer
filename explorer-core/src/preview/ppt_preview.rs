use std::path::Path;

use office_oxide::Document;

const MAX_BYTES: u64 = 32 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct PptPreview {
    pub content: String,
    pub slide_count: usize,
}

impl PptPreview {
    pub fn from_path(path: &Path) -> Result<Self, String> {
        let document = Document::open(path).map_err(|_| "preview-ppt-failed".to_string())?;
        let slide_count = document
            .as_pptx()
            .map(|ppt| ppt.slides.len())
            .or_else(|| document.as_ppt().map(|ppt| ppt.slides.len()))
            .unwrap_or(0);

        Ok(Self {
            content: document.plain_text(),
            slide_count,
        })
    }
}

pub fn is_extension(ext: &str) -> bool {
    matches!(ext, "ppt" | "pptx")
}

pub fn load_from_path(path: &Path, size: u64) -> Result<PptPreview, String> {
    if size > MAX_BYTES {
        return Err("preview-too-large".to_string());
    }

    PptPreview::from_path(path)
}
