use std::collections::BTreeSet;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use explorer_core::filesystem::{EntryKind, Mounter};
use explorer_core::FileEntry;

use crate::backend::{ID, ZipBackend};
use crate::path::{entry_name, strip_prefix, zip_prefix};

impl ZipBackend {
    pub fn read_zip_directory(
        &self,
        container: &Path,
        inner: &Path,
    ) -> Result<Vec<FileEntry>, String> {
        let file = File::open(container).map_err(|err| err.to_string())?;
        let mut zip = zip::ZipArchive::new(file).map_err(|err| err.to_string())?;
        let prefix = zip_prefix(inner);

        let mut directories = BTreeSet::new();
        let mut files = Vec::new();

        for index in 0..zip.len() {
            let entry = zip.by_index(index).map_err(|err| err.to_string())?;
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
                        Mounter::join_mounted_inner(inner, parts[0]),
                        ID,
                    ),
                    is_dir: false,
                    size: entry.size(),
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
                    container.to_path_buf(),
                    Mounter::join_mounted_inner(inner, &name),
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

    pub fn read_zip_bytes(&self, container: &Path, inner: &Path) -> Result<Vec<u8>, String> {
        if inner.as_os_str().is_empty() {
            return Err("archive-entry-required".to_string());
        }

        let zip_file = File::open(container).map_err(|err| err.to_string())?;
        let mut zip = zip::ZipArchive::new(zip_file).map_err(|err| err.to_string())?;
        let entry_name = entry_name(inner);
        let mut entry = zip.by_name(&entry_name).map_err(|err| err.to_string())?;
        let mut bytes = Vec::new();
        entry
            .read_to_end(&mut bytes)
            .map_err(|err| err.to_string())?;
        Ok(bytes)
    }

    pub fn entry_kind(&self, container: &Path, inner: &Path) -> Option<EntryKind> {
        let zip_file = std::fs::File::open(container).ok()?;
        let mut zip = zip::ZipArchive::new(zip_file).ok()?;
        let name = entry_name(inner);

        if zip.by_name(&name).is_ok() {
            return Some(EntryKind::File);
        }

        let prefix = if name.is_empty() {
            String::new()
        } else {
            format!("{name}/")
        };

        for index in 0..zip.len() {
            let entry = zip.by_index(index).ok()?;
            let entry_name = entry.name().replace('\\', "/");
            if prefix.is_empty() {
                if !entry_name.is_empty() {
                    return Some(EntryKind::Directory);
                }
                continue;
            }
            if entry_name.starts_with(&prefix) {
                return Some(EntryKind::Directory);
            }
        }

        None
    }

    pub fn extract_for_open(&self, container: &Path, inner: &Path) -> Result<PathBuf, String> {
        let temp_dir = std::env::temp_dir().join("explorer-archive-preview");
        std::fs::create_dir_all(&temp_dir).map_err(|err| err.to_string())?;
        let file_name = inner
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| "preview.bin".to_string());
        let output = temp_dir.join(file_name);
        std::fs::write(&output, self.read_zip_bytes(container, inner)?)
            .map_err(|err| err.to_string())?;
        Ok(output)
    }
}
