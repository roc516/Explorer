use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use explorer_core::{DirEntry, FsEntry};
use zip::ZipArchive;

use crate::archive::ZipEntryRecord;
use crate::file::ZipFile;
use crate::path::{join_dir_name, strip_prefix, zip_prefix};
use crate::reader::BlockReader;

/// A directory inside a zip — listed via [`DirEntry`], not [`MountedFs`].
struct ZipDir {
    name: String,
    path: PathBuf,
    entries: Arc<Vec<ZipEntryRecord>>,
    archive: Arc<Mutex<ZipArchive<BlockReader>>>,
    dir: String,
}

pub(crate) fn read_directory(
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
            files.push(FsEntry::File(Arc::new(ZipFile {
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
