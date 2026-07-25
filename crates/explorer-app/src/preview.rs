mod hex_preview;
mod image_preview;
mod pdf_preview;
mod ppt_preview;
mod text_preview;
mod word_preview;

use std::path::Path;

use explorer_core::FsEntry;

pub use hex_preview::HexPreview;
pub use image_preview::ImagePreview;
pub use pdf_preview::PdfPreview;
pub use ppt_preview::PptPreview;
pub use text_preview::{needs_reindex, TextEncoding, TextPreview};
pub use word_preview::WordPreview;

#[derive(Debug, Clone)]
pub enum PreviewKind {
    Text(TextPreview),
    Image(ImagePreview),
    Word(WordPreview),
    Ppt(PptPreview),
    Pdf(PdfPreview),
    Hex(HexPreview),
    Unsupported { extension: Option<String> },
}

#[derive(Debug, Clone)]
pub struct PreviewFile {
    pub name: String,
    pub size: u64,
    pub kind: PreviewKind,
}

/// Load preview metadata / content from a listed [`FsEntry`] (no path re-resolve).
///
/// Text and hex keep a content handle and load on demand; other previewers may
/// reject oversized files by declared size, then read when within their limit.
pub fn load_preview(entry: &FsEntry) -> Result<PreviewFile, String> {
    let FsEntry::File(file) = entry else {
        return Err("preview-not-file".to_string());
    };
    let name = file.name.clone();
    if name.is_empty() {
        return Err("preview-not-file".to_string());
    }

    let extension = extension_of(&name);
    let size = file.size;

    let kind = match extension.as_deref() {
        Some(ext) if text_preview::is_extension(ext) => {
            PreviewKind::Text(text_preview::load(file)?)
        }
        Some(ext) if image_preview::is_extension(ext) => {
            PreviewKind::Image(image_preview::load(&mut *file.open()?, size)?)
        }
        Some(ext) if word_preview::is_extension(ext) => {
            PreviewKind::Word(word_preview::load(&mut *file.open()?, size, ext)?)
        }
        Some(ext) if ppt_preview::is_extension(ext) => {
            PreviewKind::Ppt(ppt_preview::load(&mut *file.open()?, size, ext)?)
        }
        Some(ext) if pdf_preview::is_extension(ext) => {
            PreviewKind::Pdf(pdf_preview::load(&mut *file.open()?, size)?)
        }
        _ => PreviewKind::Hex(hex_preview::load(file)),
    };

    Ok(PreviewFile { name, size, kind })
}

/// Open the file with the system default app.
///
/// Absolute disk paths are opened in place; archive entries are written to a
/// temp file first (using the listed content handle).
pub fn open_with_system(entry: &FsEntry) -> Result<(), String> {
    let FsEntry::File(file) = entry else {
        return Err("preview-not-file".to_string());
    };

    let path = if file.path.is_absolute() {
        file.path.clone()
    } else {
        let temp_dir = std::env::temp_dir().join("explorer-archive-preview");
        std::fs::create_dir_all(&temp_dir).map_err(|err| err.to_string())?;
        let file_name = if file.name.is_empty() {
            "preview.bin".to_string()
        } else {
            file.name.clone()
        };
        let output = temp_dir.join(file_name);
        let mut reader = file.open()?;
        let mut output_file =
            std::fs::File::create(&output).map_err(|err| err.to_string())?;
        std::io::copy(&mut reader, &mut output_file).map_err(|err| err.to_string())?;
        output
    };

    open::that(&path).map_err(|err| err.to_string())
}

pub fn is_previewable(entry: &FsEntry) -> bool {
    matches!(entry, FsEntry::File(_))
}

fn extension_of(name: &str) -> Option<String> {
    Path::new(name)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(str::to_ascii_lowercase)
}
