use std::io::{Cursor, Read};

use super::io;

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

pub fn load(reader: &mut dyn Read, size: u64) -> Result<ImagePreview, String> {
    let bytes = io::copy_limited(reader, MAX_BYTES, Some(size))?;
    let (width, height) = image::ImageReader::new(Cursor::new(&bytes))
        .with_guessed_format()
        .map_err(|err| err.to_string())?
        .into_dimensions()
        .map_err(|err| err.to_string())?;

    Ok(ImagePreview {
        bytes,
        width,
        height,
    })
}
