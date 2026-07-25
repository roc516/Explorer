use std::collections::BTreeSet;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::{Arc, Mutex};

use explorer_core::filesystem::{BlockDevice, BlockIo, FsBackend, MountedDevice};
use explorer_core::{DirEntry, Directory, FileBytes, FileEntry as CoreFileEntry, FsEntry};
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
    archive: Arc<Mutex<ZipArchive<BlockReader>>>,
    index: usize,
}

impl FileBytes for ZipFileBytes {
    fn open(&self) -> Result<Box<dyn Read + Send>, String> {
        let archive = self.archive.clone();
        let index = self.index;
        let (tx, rx) = std::sync::mpsc::sync_channel::<Vec<u8>>(2);
        std::thread::Builder::new()
            .name("zip-entry-read".into())
            .spawn(move || {
                let Ok(mut archive) = archive.lock() else {
                    return;
                };
                let Ok(mut entry) = archive.by_index(index) else {
                    return;
                };
                let mut buf = vec![0u8; 64 * 1024];
                loop {
                    match entry.read(&mut buf) {
                        Ok(0) => break,
                        Ok(n) => {
                            if tx.send(buf[..n].to_vec()).is_err() {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }
            })
            .map_err(|err| err.to_string())?;
        Ok(Box::new(ChunkReader {
            rx,
            current: Vec::new(),
            offset: 0,
        }))
    }
}

/// Forwards chunks from a background zip decompress thread.
struct ChunkReader {
    rx: std::sync::mpsc::Receiver<Vec<u8>>,
    current: Vec<u8>,
    offset: usize,
}

impl Read for ChunkReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        while self.offset >= self.current.len() {
            match self.rx.recv() {
                Ok(chunk) => {
                    self.current = chunk;
                    self.offset = 0;
                }
                Err(_) => return Ok(0),
            }
        }
        let available = self.current.len() - self.offset;
        let n = available.min(buf.len());
        buf[..n].copy_from_slice(&self.current[self.offset..self.offset + n]);
        self.offset += n;
        Ok(n)
    }
}

/// Mount root of a zip archive.
pub struct ZipFs {
    entries: Arc<Vec<ZipEntryRecord>>,
    archive: Arc<Mutex<ZipArchive<BlockReader>>>,
}

/// A directory inside a zip — listed via [`Directory`], not [`MountedDevice`].
struct ZipDir {
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
            files.push(FsEntry::File(CoreFileEntry::new(
                parts[0].to_string(),
                join_dir_name(dir, parts[0]),
                entry.size,
                None,
                Arc::new(ZipFileBytes {
                    archive: archive.clone(),
                    index: entry.index,
                }),
            )));
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
            FsEntry::Dir(DirEntry::new(
                name.clone(),
                join_dir_name(dir, &name),
                Arc::new(ZipDir {
                    entries: entries.clone(),
                    archive: archive.clone(),
                    dir: child_dir,
                }),
            ))
        })
        .collect();

    items.append(&mut files);
    items.sort_by(|left, right| {
        let left_is_dir = matches!(left, FsEntry::Dir(_));
        let right_is_dir = matches!(right, FsEntry::Dir(_));
        let left_name = match left {
            FsEntry::Dir(d) => &d.name,
            FsEntry::File(f) => &f.name,
        };
        let right_name = match right {
            FsEntry::Dir(d) => &d.name,
            FsEntry::File(f) => &f.name,
        };
        match (left_is_dir, right_is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => left_name.to_lowercase().cmp(&right_name.to_lowercase()),
        }
    });

    Ok(items)
}

impl MountedDevice for ZipFs {
    fn list(&self) -> Result<Vec<FsEntry>, String> {
        read_directory(&self.entries, &self.archive, "")
    }
}

impl Directory for ZipDir {
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

    fn mount(&self, device: &BlockDevice) -> Result<Box<dyn MountedDevice>, String> {
        ZipFs::open(device).map(|fs| Box::new(fs) as Box<dyn MountedDevice>)
    }
}
