use std::fs;
use std::path::Path;

use explorer_core::{DirEntry, FileEntry, FsEntry};
use explorer_core::filesystem::disk_path;

pub fn read_directory(backend_id: &'static str, dir: &Path) -> Result<Vec<FsEntry>, String> {
    let entries = fs::read_dir(dir).map_err(|err| err.to_string())?;

    let mut items = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|err| err.to_string())?;
        let metadata = entry.metadata().map_err(|err| err.to_string())?;
        let file_type = entry.file_type().map_err(|err| err.to_string())?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let path = disk_path(entry.path(), backend_id);

        if file_type.is_dir() {
            items.push(FsEntry::Dir(DirEntry { name, path }));
        } else {
            items.push(FsEntry::File(FileEntry {
                name,
                path,
                size: metadata.len(),
                modified: metadata.modified().ok(),
            }));
        }
    }

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
