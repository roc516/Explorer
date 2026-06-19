use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use crate::text_encoding::TextPreview;

const MAX_TEXT_BYTES: u64 = 512 * 1024;
const MAX_IMAGE_BYTES: u64 = 8 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct ImagePreview {
    pub bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub enum PreviewKind {
    Text(TextPreview),
    Image(ImagePreview),
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

    match extension.as_deref() {
        Some(ext) if is_text_extension(ext) => load_text(path, name, size),
        Some(ext) if is_image_extension(ext) => load_image(path, name, size),
        _ => Ok(PreviewFile {
            path: path.to_path_buf(),
            name,
            size,
            kind: PreviewKind::Unsupported { extension },
        }),
    }
}

fn load_text(path: &Path, name: String, size: u64) -> Result<PreviewFile, String> {
    if size > MAX_TEXT_BYTES {
        return Err("preview-too-large".to_string());
    }

    let bytes = fs::read(path).map_err(|err| err.to_string())?;
    let text = TextPreview::from_bytes(bytes)?;

    Ok(PreviewFile {
        path: path.to_path_buf(),
        name,
        size,
        kind: PreviewKind::Text(text),
    })
}

fn load_image(path: &Path, name: String, size: u64) -> Result<PreviewFile, String> {
    if size > MAX_IMAGE_BYTES {
        return Err("preview-too-large".to_string());
    }

    let bytes = fs::read(path).map_err(|err| err.to_string())?;
    let (width, height) = image_dimensions(&bytes)?;

    Ok(PreviewFile {
        path: path.to_path_buf(),
        name,
        size,
        kind: PreviewKind::Image(ImagePreview {
            bytes,
            width,
            height,
        }),
    })
}

fn image_dimensions(bytes: &[u8]) -> Result<(u32, u32), String> {
    image::ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|err| err.to_string())?
        .into_dimensions()
        .map_err(|err| err.to_string())
}

fn is_text_extension(ext: &str) -> bool {
    matches!(
        ext,
        "txt"
            | "md"
            | "markdown"
            | "json"
            | "xml"
            | "html"
            | "htm"
            | "css"
            | "js"
            | "ts"
            | "tsx"
            | "jsx"
            | "rs"
            | "toml"
            | "yaml"
            | "yml"
            | "ftl"
            | "log"
            | "ini"
            | "cfg"
            | "conf"
            | "csv"
            | "sql"
            | "sh"
            | "bat"
            | "ps1"
            | "c"
            | "cpp"
            | "h"
            | "hpp"
            | "cs"
            | "go"
            | "java"
            | "kt"
            | "py"
            | "rb"
            | "php"
            | "swift"
            | "zig"
            | "lua"
            | "env"
    )
}

fn is_image_extension(ext: &str) -> bool {
    matches!(
        ext,
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "ico"
    )
}
