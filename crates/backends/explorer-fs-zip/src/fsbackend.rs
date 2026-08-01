use std::collections::BTreeSet;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use explorer_core::{BlockDevice, BlockIo};
use explorer_core::filesystem::{FsBackend, MountedFs};
use explorer_core::{DirEntry, FileEntry, FsEntry, SeekRead};
use zip::ZipArchive;

use crate::path::{join_dir_name, strip_prefix, zip_prefix};

struct ZipEntryRecord {
    name: String,
    size: u64,
    index: usize,
}

/// Adapts [`BlockIo`] to `Read + Seek` for [`ZipArchive`].
struct BlockReader {
    io: Arc<dyn BlockIo>,
    pos: u64,
}

impl BlockReader {
    fn new(io: Arc<dyn BlockIo>) -> Self {
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

struct ZipFileBytes {
    name: String,
    path: PathBuf,
    size: u64,
    modified: Option<SystemTime>,
    archive: Arc<Mutex<ZipArchive<BlockReader>>>,
    index: usize,
}

impl FileEntry for ZipFileBytes {
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

impl ZipFileBytes {
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
struct ZipSeekReader {
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

/// Mount root of a zip archive.
pub struct ZipFs {
    entries: Arc<Vec<ZipEntryRecord>>,
    archive: Arc<Mutex<ZipArchive<BlockReader>>>,
}

/// A directory inside a zip — listed via [`DirEntry`], not [`MountedFs`].
struct ZipDir {
    name: String,
    path: PathBuf,
    entries: Arc<Vec<ZipEntryRecord>>,
    archive: Arc<Mutex<ZipArchive<BlockReader>>>,
    dir: String,
}

impl ZipFs {
    pub fn open(device: &BlockDevice) -> Result<Self, String> {
        let reader = BlockReader::new(device.io().clone());
        let mut archive = ZipArchive::new(reader).map_err(|err| err.to_string())?;
        let mut entries = Vec::with_capacity(archive.len());

        for index in 0..archive.len() {
            let entry = archive.by_index(index).map_err(|err| err.to_string())?;
            let name = entry.name().replace('\\', "/");
            if name.ends_with('/') {
                continue;
            }
            entries.push(ZipEntryRecord {
                name,
                size: entry.size(),
                index,
            });
        }

        Ok(Self {
            entries: Arc::new(entries),
            archive: Arc::new(Mutex::new(archive)),
        })
    }
}

fn read_directory(
    entries: &Arc<Vec<ZipEntryRecord>>,
    archive: &Arc<Mutex<ZipArchive<BlockReader>>>,
    dir: &str,
) -> Result<Vec<FsEntry>, String> {
    let prefix = zip_prefix(dir);
    let mut directories = BTreeSet::new();
    let mut files = Vec::new();

    for entry in entries.iter() {
        let Some(relative) = strip_prefix(&entry.name, &prefix) else {
            continue;
        };
        if relative.is_empty() {
            continue;
        }

        let parts: Vec<&str> = relative.split('/').collect();
        if parts.len() == 1 {
            files.push(FsEntry::File(Arc::new(ZipFileBytes {
                name: parts[0].to_string(),
                path: join_dir_name(dir, parts[0]),
                size: entry.size,
                modified: None,
                archive: archive.clone(),
                index: entry.index,
            })));
        } else {
            directories.insert(parts[0].to_string());
        }
    }

    let mut items: Vec<FsEntry> = directories
        .into_iter()
        .map(|name| {
            let child_dir = if dir.is_empty() {
                name.clone()
            } else {
                format!("{dir}/{name}")
            };
            FsEntry::Dir(
                Arc::new(ZipDir {
                    name: name.clone(),
                    path: join_dir_name(dir, &name),
                    entries: entries.clone(),
                    archive: archive.clone(),
                    dir: child_dir,
                }) as Arc<dyn DirEntry>,
            )
        })
        .collect();

    items.append(&mut files);
    items.sort_by(|left, right| {
        let left_is_dir = matches!(left, FsEntry::Dir(_));
        let right_is_dir = matches!(right, FsEntry::Dir(_));
        let left_name: &str = match left {
            FsEntry::Dir(d) => d.name(),
            FsEntry::File(f) => f.name(),
            FsEntry::Volume(v) => v.name(),
        };
        let right_name: &str = match right {
            FsEntry::Dir(d) => d.name(),
            FsEntry::File(f) => f.name(),
            FsEntry::Volume(v) => v.name(),
        };
        match (left_is_dir, right_is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => left_name.to_lowercase().cmp(&right_name.to_lowercase()),
        }
    });

    Ok(items)
}

impl MountedFs for ZipFs {
    fn list(&self) -> Result<Vec<FsEntry>, String> {
        read_directory(&self.entries, &self.archive, "")
    }
}

impl DirEntry for ZipDir {
    fn name(&self) -> &str {
        &self.name
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn list(&self) -> Result<Vec<FsEntry>, String> {
        read_directory(&self.entries, &self.archive, &self.dir)
    }
}

fn extension_matches(name: &str) -> bool {
    Path::new(name)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| crate::EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

fn looks_like_zip(device: &BlockDevice) -> bool {
    let mut magic = [0u8; 4];
    match device.read_at(0, &mut magic) {
        Ok(n) if n >= 2 => {}
        _ => return false,
    }
    matches!(&magic[..2], b"PK")
        && (magic[2] == 0x03 || magic[2] == 0x05 || magic[2] == 0x07)
}

impl FsBackend for crate::ZipBackend {
    fn id(&self) -> &'static str {
        crate::ID
    }

    fn matches(&self, device: &BlockDevice) -> bool {
        if extension_matches(device.name()) {
            return true;
        }
        looks_like_zip(device)
    }

    fn mount(&self, device: &BlockDevice) -> Result<Box<dyn MountedFs>, String> {
        ZipFs::open(device).map(|fs| Box::new(fs) as Box<dyn MountedFs>)
    }
}
