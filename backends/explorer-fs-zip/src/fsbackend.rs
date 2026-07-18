use std::collections::BTreeSet;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::{Arc, Mutex};

use explorer_core::filesystem::{
    BlockDevice, BlockIo, DeviceId, EntryKind, FsBackend, MountedDevice, Mounter,
};
use explorer_core::{DirEntry, FileEntry as CoreFileEntry, FsEntry};
use zip::ZipArchive;

use crate::path::{entry_name, strip_prefix, zip_prefix};

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

pub struct ZipFs {
    device_id: DeviceId,
    entries: Vec<ZipEntryRecord>,
    archive: Mutex<ZipArchive<BlockReader>>,
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
            device_id: device.id().clone(),
            entries,
            archive: Mutex::new(archive),
        })
    }

    fn read_directory(&self, inner: &Path) -> Result<Vec<FsEntry>, String> {
        let prefix = zip_prefix(inner);
        let mut directories = BTreeSet::new();
        let mut files = Vec::new();

        for entry in &self.entries {
            let Some(relative) = strip_prefix(&entry.name, &prefix) else {
                continue;
            };
            if relative.is_empty() {
                continue;
            }

            let parts: Vec<&str> = relative.split('/').collect();
            if parts.len() == 1 {
                files.push(FsEntry::File(CoreFileEntry {
                    name: parts[0].to_string(),
                    path: Mounter::mount_path(
                        self.device_id.clone(),
                        Mounter::join_mounted_path(inner, parts[0]),
                        crate::ID,
                    ),
                    size: entry.size,
                    modified: None,
                }));
            } else {
                directories.insert(parts[0].to_string());
            }
        }

        let mut items: Vec<FsEntry> = directories
            .into_iter()
            .map(|name| {
                FsEntry::Dir(DirEntry {
                    path: Mounter::mount_path(
                        self.device_id.clone(),
                        Mounter::join_mounted_path(inner, &name),
                        crate::ID,
                    ),
                    name,
                })
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

    fn read_bytes(&self, inner: &Path) -> Result<Vec<u8>, String> {
        if inner.as_os_str().is_empty() {
            return Err("archive-entry-required".to_string());
        }

        let entry_name = entry_name(inner);
        let idx = self
            .entries
            .iter()
            .find(|entry| entry.name == entry_name)
            .map(|entry| entry.index)
            .ok_or_else(|| "archive-entry-not-found".to_string())?;

        let mut archive = self
            .archive
            .lock()
            .map_err(|_| "archive-lock-poisoned".to_string())?;
        let mut entry = archive.by_index(idx).map_err(|err| err.to_string())?;
        let mut bytes = Vec::new();
        entry
            .read_to_end(&mut bytes)
            .map_err(|err| err.to_string())?;
        Ok(bytes)
    }
}

impl MountedDevice for ZipFs {
    fn list(&self, path: &Path) -> Result<Vec<FsEntry>, String> {
        self.read_directory(path)
    }

    fn read(&self, path: &Path) -> Result<Vec<u8>, String> {
        self.read_bytes(path)
    }

    fn exists(&self, path: &Path) -> bool {
        if path.as_os_str().is_empty() {
            return !self.entries.is_empty();
        }
        let needle = entry_name(path);
        let prefix = format!("{needle}/");
        self.entries
            .iter()
            .any(|entry| entry.name == needle || entry.name.starts_with(&prefix))
    }

    fn entry_kind(&self, path: &Path) -> Option<EntryKind> {
        let name = entry_name(path);
        if name.is_empty() {
            return (!self.entries.is_empty()).then_some(EntryKind::Directory);
        }
        if self.entries.iter().any(|entry| entry.name == name) {
            return Some(EntryKind::File);
        }
        let prefix = format!("{name}/");
        self.entries
            .iter()
            .any(|entry| entry.name.starts_with(&prefix))
            .then_some(EntryKind::Directory)
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
    // Local file header or empty archive EOCD
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
