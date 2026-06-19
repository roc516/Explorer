use std::io::{copy, Read};

pub fn copy_limited(
    reader: &mut dyn Read,
    max_bytes: u64,
    size_hint: Option<u64>,
) -> Result<Vec<u8>, String> {
    if let Some(size) = size_hint {
        if size > max_bytes {
            return Err("preview-too-large".to_string());
        }
    }

    let mut buffer = Vec::new();
    if let Some(size) = size_hint {
        buffer.reserve(size.min(max_bytes) as usize);
    }

    let mut limited = reader.take(max_bytes.saturating_add(1));
    let copied = copy(&mut limited, &mut buffer).map_err(|err| err.to_string())?;
    if copied > max_bytes {
        return Err("preview-too-large".to_string());
    }

    Ok(buffer)
}
