mod image_preview;
mod io;
mod pdf_preview;
mod ppt_preview;
mod text_preview;
mod word_preview;

use std::fs::{self, File};
use std::io::Read;
use std::path::PathBuf;

use crate::archive;
use crate::browse_path::BrowsePath;

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

pub fn load_preview(path: &BrowsePath) -> Result<PreviewFile, String> {
    match path {
        BrowsePath::Local(local) => {
            let metadata = fs::metadata(local).map_err(|err| err.to_string())?;
            if metadata.is_dir() {
                return Err("preview-not-file".to_string());
            }

            let mut file = File::open(local).map_err(|err| err.to_string())?;
            read_preview_file(path, &mut file, metadata.len())
        }
        BrowsePath::Archive { .. } => archive::with_entry_reader(path, |reader, size| {
            read_preview_file(path, reader, size)
        }),
    }
}

fn read_preview_file(
    path: &BrowsePath,
    reader: &mut dyn Read,
    size: u64,
) -> Result<PreviewFile, String> {
    let name = path.file_name();
    if name.is_empty() {
        return Err("preview-not-file".to_string());
    }

    let extension = path.extension();
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
        path: path.preview_path(),
        name,
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

pub fn is_previewable(path: &BrowsePath) -> bool {
    path.extension()
        .as_deref()
        .is_some_and(is_previewable_extension)
}
