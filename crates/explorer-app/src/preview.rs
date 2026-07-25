mod image_preview;
mod io;
mod pdf_preview;
mod ppt_preview;
mod text_preview;
mod word_preview;

use std::io::{Cursor, Read};
use std::path::Path;

use explorer_core::FsEntry;

pub use image_preview::ImagePreview;
pub use pdf_preview::PdfPreview;
pub use ppt_preview::PptPreview;
pub use text_preview::{TextEncoding, TextPreview};
pub use word_preview::WordPreview;

#[derive(Debug, Clone)]
pub enum PreviewKind {
    Text(TextPreview),
    Image(ImagePreview),
    Word(WordPreview),
    Ppt(PptPreview),
    Pdf(PdfPreview),
    Unsupported { extension: Option<String> },
}

#[derive(Debug, Clone)]
pub struct PreviewFile {
    pub name: String,
    pub size: u64,
    pub kind: PreviewKind,
}

/// Load preview content from a listed [`FsEntry`] (no path re-resolve).
pub fn load_preview(entry: &FsEntry) -> Result<PreviewFile, String> {
    let FsEntry::File(file) = entry else {
        return Err("preview-not-file".to_string());
    };
    let name = file.name.clone();
    let extension = extension_of(&name);
    let bytes = file.read()?;
    let size = bytes.len() as u64;
    read_preview_file(&name, extension, &mut Cursor::new(bytes), size)
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
        std::fs::write(&output, file.read()?).map_err(|err| err.to_string())?;
        output
    };

    open::that(&path).map_err(|err| err.to_string())
}

fn read_preview_file(
    name: &str,
    extension: Option<String>,
    reader: &mut dyn Read,
    size: u64,
) -> Result<PreviewFile, String> {
    if name.is_empty() {
        return Err("preview-not-file".to_string());
    }

    let kind = match extension.as_deref() {
        Some(ext) if text_preview::is_extension(ext) => {
            PreviewKind::Text(text_preview::load(reader, size)?)
        }
        Some(ext) if image_preview::is_extension(ext) => {
            PreviewKind::Image(image_preview::load(reader, size)?)
        }
        Some(ext) if word_preview::is_extension(ext) => {
            PreviewKind::Word(word_preview::load(reader, size, ext)?)
        }
        Some(ext) if ppt_preview::is_extension(ext) => {
            PreviewKind::Ppt(ppt_preview::load(reader, size, ext)?)
        }
        Some(ext) if pdf_preview::is_extension(ext) => {
            PreviewKind::Pdf(pdf_preview::load(reader, size)?)
        }
        _ => PreviewKind::Unsupported { extension },
    };

    Ok(PreviewFile {
        name: name.to_string(),
        size,
        kind,
    })
}

pub fn is_previewable_extension(ext: &str) -> bool {
    text_preview::is_extension(ext)
        || image_preview::is_extension(ext)
        || word_preview::is_extension(ext)
        || ppt_preview::is_extension(ext)
        || pdf_preview::is_extension(ext)
}

pub fn is_previewable(entry: &FsEntry) -> bool {
    match entry {
        FsEntry::File(file) => {
            extension_of(&file.name).is_some_and(|ext| is_previewable_extension(&ext))
        }
        FsEntry::Dir(_) => false,
    }
}

fn extension_of(name: &str) -> Option<String> {
    Path::new(name)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(str::to_ascii_lowercase)
}
