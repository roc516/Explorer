use std::collections::BTreeSet;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use explorer_core::filesystem::{EntryKind, FsBackend, MountedDevice, Mounter};
use explorer_core::{DirEntry, FileEntry as CoreFileEntry, FsEntry};
use zip::ZipArchive;

use crate::path::{entry_name, strip_prefix, zip_prefix};

struct ZipEntryRecord {
    name: String,
    size: u64,
    index: usize,
}

pub struct ZipFs {
    container: PathBuf,
    entries: Vec<ZipEntryRecord>,
    archive: Mutex<ZipArchive<File>>,
}

impl ZipFs {
    pub fn open(container: &Path) -> Result<Self, String> {
        let file = File::open(container).map_err(|err| err.to_string())?;
        let mut archive = ZipArchive::new(file).map_err(|err| err.to_string())?;
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
            container: container.to_path_buf(),
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
                        self.container.clone(),
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
            .map(|name| FsEntry::Dir(DirEntry {
                path: Mounter::mount_path(
                    self.container.clone(),
                    Mounter::join_mounted_path(inner, &name),
                    crate::ID,
                ),
                name,
            }))
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
        // Need to handle seeking: the Mutex<ZipArchive> is reused across reads
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

impl FsBackend for crate::ZipBackend {
    fn id(&self) -> &'static str {
        crate::ID
    }

    fn matches(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| crate::EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()))
            .unwrap_or(false)
    }

    fn mount(&self, path: &Path) -> Result<Box<dyn MountedDevice>, String> {
        ZipFs::open(path).map(|fs| Box::new(fs) as Box<dyn MountedDevice>)
    }
}
