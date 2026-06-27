use std::collections::BTreeSet;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use explorer_core::filesystem::{EntryKind, EPath, FsBackend, Mounter};
use explorer_core::FileEntry;
use zip::ZipArchive;

use crate::path::{entry_name, strip_prefix, zip_prefix};

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

    fn exists(&self, path: &EPath) -> bool {
        let Ok((container, inner)) = Mounter::mount_ref(path) else {
            return false;
        };
        if !container.is_file() {
            return false;
        }
        if inner.as_os_str().is_empty() {
            return true;
        }

        let file = match File::open(container) {
            Ok(f) => f,
            Err(_) => return false,
        };
        let mut archive = match ZipArchive::new(file) {
            Ok(a) => a,
            Err(_) => return false,
        };

        let needle = entry_name(inner);
        let prefix = format!("{needle}/");

        for i in 0..archive.len() {
            let Ok(entry) = archive.by_index(i) else { continue };
            let name = entry.name().replace('\\', "/");
            if name == needle {
                return true; // exact file match
            }
            if name.starts_with(&prefix) {
                return true; // directory with children
            }
        }
        false
    }

    fn kind(&self, container: &Path, inner: &Path) -> Option<EntryKind> {
        let file = File::open(container).ok()?;
        let mut archive = ZipArchive::new(file).ok()?;
        let name = entry_name(inner);

        if name.is_empty() {
            // root of the archive always exists as directory
            return if archive.len() > 0 {
                Some(EntryKind::Directory)
            } else {
                None
            };
        }

        // check for exact file match
        for i in 0..archive.len() {
            let Ok(entry) = archive.by_index(i) else { continue };
            if entry.name().replace('\\', "/") == name {
                return Some(EntryKind::File);
            }
        }

        // check for directory (has children with this prefix)
        let prefix = format!("{name}/");
        for i in 0..archive.len() {
            let Ok(entry) = archive.by_index(i) else { continue };
            if entry.name().replace('\\', "/").starts_with(&prefix) {
                return Some(EntryKind::Directory);
            }
        }

        None
    }

    fn list(&self, path: &EPath) -> Result<Vec<FileEntry>, String> {
        let (container, inner) = Mounter::mount_ref(path)?;
        let file = File::open(container).map_err(|err| err.to_string())?;
        let mut archive = ZipArchive::new(file).map_err(|err| err.to_string())?;

        let prefix = zip_prefix(inner);
        let mut directories = BTreeSet::new();
        let mut files = Vec::new();

        for i in 0..archive.len() {
            let entry = archive.by_index(i).map_err(|err| err.to_string())?;
            let name = entry.name().replace('\\', "/");
            if name.ends_with('/') {
                continue;
            }

            let Some(relative) = strip_prefix(&name, &prefix) else {
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
                        container.to_path_buf(),
                        Mounter::join_mounted_path(inner, parts[0]),
                        crate::ID,
                    ),
                    is_dir: false,
                    size: entry.size(),
                    modified: None,
                });
            } else {
                directories.insert(parts[0].to_string());
            }
        }

        let mut items: Vec<FileEntry> = directories
            .into_iter()
            .map(|name| FileEntry {
                path: Mounter::mount_path(
                    container.to_path_buf(),
                    Mounter::join_mounted_path(inner, &name),
                    crate::ID,
                ),
                name,
                is_dir: true,
                size: 0,
                modified: None,
            })
            .collect();

        items.append(&mut files);
        items.sort_by(|left, right| match (left.is_dir, right.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => left.name.to_lowercase().cmp(&right.name.to_lowercase()),
        });

        Ok(items)
    }

    fn read(&self, path: &EPath) -> Result<Vec<u8>, String> {
        let (container, inner) = Mounter::mount_ref(path)?;
        if inner.as_os_str().is_empty() {
            return Err("archive-entry-required".to_string());
        }

        let name = entry_name(inner);
        let file = File::open(container).map_err(|err| err.to_string())?;
        let mut archive = ZipArchive::new(file).map_err(|err| err.to_string())?;
        let mut entry = archive.by_name(&name).map_err(|err| err.to_string())?;
        let mut bytes = Vec::new();
        entry
            .read_to_end(&mut bytes)
            .map_err(|err| err.to_string())?;
        Ok(bytes)
    }
}
