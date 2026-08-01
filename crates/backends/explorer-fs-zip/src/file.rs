use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use explorer_core::{FileEntry, SeekRead};
use zip::ZipArchive;

use crate::reader::BlockReader;

pub(crate) struct ZipFile {
    pub(crate) name: String,
    pub(crate) path: PathBuf,
    pub(crate) size: u64,
    pub(crate) modified: Option<SystemTime>,
    pub(crate) archive: Arc<Mutex<ZipArchive<BlockReader>>>,
    pub(crate) index: usize,
}

impl FileEntry for ZipFile {
    fn name(&self) -> &str {
        &self.name
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn size(&self) -> u64 {
        self.size
    }

    fn modified(&self) -> Option<SystemTime> {
        self.modified
    }

    fn open(&self) -> Result<Box<dyn SeekRead>, String> {
        Ok(Box::new(self.seek_reader()))
    }
}

impl ZipFile {
    fn seek_reader(&self) -> ZipSeekReader {
        ZipSeekReader {
            archive: self.archive.clone(),
            index: self.index,
            size: self.size,
            pos: 0,
            cache: Vec::new(),
            cache_off: 0,
        }
    }
}

/// Seekable reader over a zip entry. Seeking is supported by restarting the
/// entry stream and skipping to the target offset (with a small read-ahead cache).
pub(crate) struct ZipSeekReader {
    archive: Arc<Mutex<ZipArchive<BlockReader>>>,
    index: usize,
    size: u64,
    pos: u64,
    cache: Vec<u8>,
    cache_off: usize,
}

impl ZipSeekReader {
    fn fill_cache(&mut self) -> io::Result<()> {
        self.cache.clear();
        self.cache_off = 0;
        if self.pos >= self.size {
            return Ok(());
        }

        let mut archive = self
            .archive
            .lock()
            .map_err(|_| io::Error::other("archive-lock-poisoned"))?;
        let mut entry = archive
            .by_index(self.index)
            .map_err(|err| io::Error::other(err))?;
        io::copy(&mut entry.by_ref().take(self.pos), &mut io::sink())?;

        let remain = (self.size - self.pos) as usize;
        let mut chunk = vec![0u8; remain.min(64 * 1024)];
        let n = entry.read(&mut chunk)?;
        chunk.truncate(n);
        self.cache = chunk;
        Ok(())
    }
}

impl Read for ZipSeekReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.pos >= self.size {
            return Ok(0);
        }
        if self.cache_off >= self.cache.len() {
            self.fill_cache()?;
            if self.cache.is_empty() {
                return Ok(0);
            }
        }
        let available = self.cache.len() - self.cache_off;
        let n = available.min(buf.len());
        buf[..n].copy_from_slice(&self.cache[self.cache_off..self.cache_off + n]);
        self.cache_off += n;
        self.pos += n as u64;
        Ok(n)
    }
}

impl Seek for ZipSeekReader {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let next = match pos {
            SeekFrom::Start(offset) => offset as i64,
            SeekFrom::End(offset) => self.size as i64 + offset,
            SeekFrom::Current(offset) => self.pos as i64 + offset,
        };
        if next < 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "seek before start",
            ));
        }
        let next = next as u64;
        if next != self.pos {
            self.pos = next;
            self.cache.clear();
            self.cache_off = 0;
        }
        Ok(self.pos)
    }
}
