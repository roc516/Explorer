use std::any::Any;
use std::collections::BTreeSet;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use explorer_core::filesystem::{EntryKind, MountSession, Mounter};
use explorer_core::FileEntry;
use zip::ZipArchive;

use crate::backend::ID;
use crate::path::{entry_name, strip_prefix, zip_prefix};

struct ZipEntryRecord {
    name: String,
    size: u64,
    index: usize,
}

pub struct ZipMountSession {
    container: PathBuf,
    entries: Vec<ZipEntryRecord>,
    archive: Mutex<ZipArchive<File>>,
}

impl MountSession for ZipMountSession {}

impl ZipMountSession {
    pub fn open(container: PathBuf) -> Result<Arc<Self>, String> {
        let file = File::open(&container).map_err(|err| err.to_string())?;
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

        Ok(Arc::new(Self {
            container,
            entries,
            archive: Mutex::new(archive),
        }))
    }

    pub fn read_directory(&self, inner: &Path) -> Result<Vec<FileEntry>, String> {
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
                files.push(FileEntry {
                    name: parts[0].to_string(),
                    path: Mounter::mount_path(
                        self.container.clone(),
                        Mounter::join_mounted_path(inner, parts[0]),
                        ID,
                    ),
                    is_dir: false,
                    size: entry.size,
                    modified: None,
                });
            } else {
                directories.insert(parts[0].to_string());
            }
        }

        let mut items = directories
            .into_iter()
            .map(|name| FileEntry {
                path: Mounter::mount_path(
                    self.container.clone(),
                    Mounter::join_mounted_path(inner, &name),
                    ID,
                ),
                name,
                is_dir: true,
                size: 0,
                modified: None,
            })
            .collect::<Vec<_>>();

        items.append(&mut files);
        items.sort_by(|left, right| match (left.is_dir, right.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => left.name.to_lowercase().cmp(&right.name.to_lowercase()),
        });

        Ok(items)
    }

    pub fn read_bytes(&self, inner: &Path) -> Result<Vec<u8>, String> {
        if inner.as_os_str().is_empty() {
            return Err("archive-entry-required".to_string());
        }

        let entry_name = entry_name(inner);
        let index = self
            .entries
            .iter()
            .find(|entry| entry.name == entry_name)
            .map(|entry| entry.index)
            .ok_or_else(|| "archive-entry-not-found".to_string())?;

        let mut archive = self
            .archive
            .lock()
            .map_err(|_| "archive-lock-poisoned".to_string())?;
        let mut entry = archive.by_index(index).map_err(|err| err.to_string())?;
        let mut bytes = Vec::new();
        entry
            .read_to_end(&mut bytes)
            .map_err(|err| err.to_string())?;
        Ok(bytes)
    }

    pub fn entry_kind(&self, inner: &Path) -> Option<EntryKind> {
        let name = entry_name(inner);

        if self.entries.iter().any(|entry| entry.name == name) {
            return Some(EntryKind::File);
        }

        let prefix = if name.is_empty() {
            String::new()
        } else {
            format!("{name}/")
        };

        for entry in &self.entries {
            if prefix.is_empty() {
                if !entry.name.is_empty() {
                    return Some(EntryKind::Directory);
                }
                continue;
            }
            if entry.name.starts_with(&prefix) {
                return Some(EntryKind::Directory);
            }
        }

        None
    }

    pub fn extract_for_open(&self, inner: &Path) -> Result<PathBuf, String> {
        let temp_dir = std::env::temp_dir().join("explorer-archive-preview");
        std::fs::create_dir_all(&temp_dir).map_err(|err| err.to_string())?;
        let file_name = inner
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| "preview.bin".to_string());
        let output = temp_dir.join(file_name);
        std::fs::write(&output, self.read_bytes(inner)?).map_err(|err| err.to_string())?;
        Ok(output)
    }
}

pub fn zip_session(path: &explorer_core::filesystem::EPath) -> Result<Arc<ZipMountSession>, String> {
    downcast_session(Mounter::session(path).ok_or("mount-session-missing".to_string())?)
}

pub fn zip_session_for(container: &Path) -> Result<Arc<ZipMountSession>, String> {
    downcast_session(
        Mounter::session(&Mounter::mount_path(
            container.to_path_buf(),
            PathBuf::new(),
            ID,
        ))
        .ok_or("mount-session-missing".to_string())?,
    )
}

fn downcast_session<T: Any + Send + Sync>(session: Arc<dyn MountSession>) -> Result<Arc<T>, String> {
    let any: Arc<dyn Any + Send + Sync> = session;
    any.downcast::<T>()
        .map_err(|_| "invalid-mount-session".to_string())
}
