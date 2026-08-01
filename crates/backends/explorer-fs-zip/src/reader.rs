use std::io::{self, Read, Seek, SeekFrom};
use std::sync::Arc;

use explorer_core::BlockIo;

/// Adapts [`BlockIo`] to `Read + Seek` for [`ZipArchive`].
pub(crate) struct BlockReader {
    io: Arc<dyn BlockIo>,
    pos: u64,
}

impl BlockReader {
    pub(crate) fn new(io: Arc<dyn BlockIo>) -> Self {
        Self { io, pos: 0 }
    }
}

impl Read for BlockReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.io.read_at(self.pos, buf)?;
        self.pos += n as u64;
        Ok(n)
    }
}

impl Seek for BlockReader {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let len = self.io.len();
        let next = match pos {
            SeekFrom::Start(offset) => offset as i64,
            SeekFrom::End(offset) => len as i64 + offset,
            SeekFrom::Current(offset) => self.pos as i64 + offset,
        };
        if next < 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "seek before start",
            ));
        }
        self.pos = next as u64;
        Ok(self.pos)
    }
}
