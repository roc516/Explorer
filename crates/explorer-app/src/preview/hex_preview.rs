use std::io::{Read, Seek, SeekFrom};

use explorer_core::FileEntry;

#[derive(Clone)]
pub struct HexPreview {
    pub size: u64,
    file: FileEntry,
}

impl std::fmt::Debug for HexPreview {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HexPreview")
            .field("size", &self.size)
            .field("name", &self.file.name)
            .finish_non_exhaustive()
    }
}

pub fn load(file: &FileEntry) -> HexPreview {
    HexPreview {
        size: file.size,
        file: file.clone(),
    }
}

impl HexPreview {
    /// Read a byte range for the current viewport via a seekable reader.
    pub fn read_range(&self, offset: u64, len: usize) -> Result<Vec<u8>, String> {
        if len == 0 || offset >= self.size {
            return Ok(Vec::new());
        }
        let available = (self.size - offset) as usize;
        let len = len.min(available);

        let mut reader = self.file.open()?;
        reader
            .seek(SeekFrom::Start(offset))
            .map_err(|err| err.to_string())?;
        read_exact_up_to(&mut reader, len)
    }
}

fn read_exact_up_to(reader: &mut dyn Read, len: usize) -> Result<Vec<u8>, String> {
    let mut buf = vec![0u8; len];
    let mut filled = 0usize;
    while filled < len {
        match reader.read(&mut buf[filled..]) {
            Ok(0) => break,
            Ok(n) => filled += n,
            Err(err) => return Err(err.to_string()),
        }
    }
    buf.truncate(filled);
    Ok(buf)
}
