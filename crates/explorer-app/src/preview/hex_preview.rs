use std::io::Read;

const MAX_BYTES: u64 = 512 * 1024;

#[derive(Debug, Clone)]
pub struct HexPreview {
    pub bytes: Vec<u8>,
}

pub fn load(reader: &mut dyn Read, size: u64) -> Result<HexPreview, String> {
    if size > MAX_BYTES {
        return Err("preview-too-large".to_string());
    }
    let mut bytes = Vec::with_capacity(size as usize);
    reader
        .read_to_end(&mut bytes)
        .map_err(|err| err.to_string())?;
    Ok(HexPreview { bytes })
}
