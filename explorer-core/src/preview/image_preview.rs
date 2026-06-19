use std::fs;
use std::io::Cursor;
use std::path::Path;

const MAX_BYTES: u64 = 8 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct ImagePreview {
    pub bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

pub fn is_extension(ext: &str) -> bool {
    matches!(
        ext,
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "ico"
    )
}

pub fn load_from_path(path: &Path, size: u64) -> Result<ImagePreview, String> {
    if size > MAX_BYTES {
        return Err("preview-too-large".to_string());
    }

    let bytes = fs::read(path).map_err(|err| err.to_string())?;
    let (width, height) = dimensions(&bytes)?;

    Ok(ImagePreview {
        bytes,
        width,
        height,
    })
}

fn dimensions(bytes: &[u8]) -> Result<(u32, u32), String> {
    image::ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|err| err.to_string())?
        .into_dimensions()
        .map_err(|err| err.to_string())
}
