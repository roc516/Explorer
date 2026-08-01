use std::path::{Component, Path};

use crate::entry::FsEntry;

/// A mounted filesystem root — lists only its immediate children.
///
/// Nested directories are listed via [`crate::entry::DirEntry::list`], not this trait.
pub trait MountedFs: Send + Sync {
    /// List immediate children of the mount root.
    fn list(&self) -> Result<Vec<FsEntry>, String>;
}

/// Resolve `path` relative to a mount root to a single [`FsEntry`].
///
/// Empty path is invalid — the mount root is the device itself, not an entry.
pub fn entry_at(root: &dyn MountedFs, path: &Path) -> Result<FsEntry, String> {
    let parts: Vec<String> = path
        .components()
        .filter_map(|component| match component {
            Component::Normal(name) => Some(name.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect();
    if parts.is_empty() {
        return Err("empty-entry-path".to_string());
    }

    let mut entries = root.list()?;
    for (index, part) in parts.iter().enumerate() {
        let entry = entries
            .into_iter()
            .find(|entry| match entry {
                FsEntry::Dir(dir) => dir.name == *part,
                FsEntry::File(file) => file.name == *part,
            })
            .ok_or_else(|| "entry-not-found".to_string())?;

        if index + 1 == parts.len() {
            return Ok(entry);
        }

        match entry {
            FsEntry::Dir(dir) => entries = dir.list()?,
            FsEntry::File(_) => return Err("not-a-directory".to_string()),
        }
    }

    Err("entry-not-found".to_string())
}
