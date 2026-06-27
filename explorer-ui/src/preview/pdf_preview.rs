use std::io::Read;

use super::io;

const MAX_BYTES: u64 = 32 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct PdfPreview {
    pub content: String,
    pub page_count: usize,
}

pub fn is_extension(ext: &str) -> bool {
    ext == "pdf"
}

pub fn load(reader: &mut dyn Read, size: u64) -> Result<PdfPreview, String> {
    let bytes = io::copy_limited(reader, MAX_BYTES, Some(size))?;
    let pages = pdf_extract::extract_text_from_mem_by_pages(&bytes)
        .map_err(|_| "preview-pdf-failed".to_string())?;

    Ok(PdfPreview {
        page_count: pages.len(),
        content: pages
            .into_iter()
            .filter(|page| !page.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n\n"),
    })
}
