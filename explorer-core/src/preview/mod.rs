mod image_preview;
mod pdf_preview;
mod ppt_preview;
mod text_preview;
mod word_preview;

use std::fs;
use std::path::{Path, PathBuf};

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
    pub path: PathBuf,
    pub name: String,
    pub size: u64,
    pub kind: PreviewKind,
}

pub fn load_preview(path: &Path) -> Result<PreviewFile, String> {
    let metadata = fs::metadata(path).map_err(|err| err.to_string())?;
    if metadata.is_dir() {
        return Err("preview-not-file".to_string());
    }

    let name = path
        .file_name()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_default();
    let size = metadata.len();
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase);
    let path = path.to_path_buf();

    let kind = match extension.as_deref() {
        Some(ext) if text_preview::is_extension(ext) => {
            PreviewKind::Text(text_preview::load_from_path(&path, size)?)
        }
        Some(ext) if image_preview::is_extension(ext) => {
            PreviewKind::Image(image_preview::load_from_path(&path, size)?)
        }
        Some(ext) if word_preview::is_extension(ext) => {
            PreviewKind::Word(word_preview::load_from_path(&path, size)?)
        }
        Some(ext) if ppt_preview::is_extension(ext) => {
            PreviewKind::Ppt(ppt_preview::load_from_path(&path, size)?)
        }
        Some(ext) if pdf_preview::is_extension(ext) => {
            PreviewKind::Pdf(pdf_preview::load_from_path(&path, size)?)
        }
        _ => PreviewKind::Unsupported { extension },
    };

    Ok(PreviewFile {
        path,
        name,
        size,
        kind,
    })
}
