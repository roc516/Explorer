use std::path::Path;

const MAX_BYTES: u64 = 32 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct PdfPreview {
    pub content: String,
    pub page_count: usize,
}

impl PdfPreview {
    pub fn from_path(path: &Path) -> Result<Self, String> {
        let pages =
            pdf_extract::extract_text_by_pages(path).map_err(|_| "preview-pdf-failed".to_string())?;
        let page_count = pages.len();
        let content = pages
            .into_iter()
            .filter(|page| !page.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n\n");

        Ok(Self {
            content,
            page_count,
        })
    }
}

pub fn is_extension(ext: &str) -> bool {
    ext == "pdf"
}

pub fn load_from_path(path: &Path, size: u64) -> Result<PdfPreview, String> {
    if size > MAX_BYTES {
        return Err("preview-too-large".to_string());
    }

    PdfPreview::from_path(path)
}
