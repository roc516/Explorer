use std::fs;
use std::path::Path;

use explorer_core::filesystem::disk_path;
use explorer_core::FileEntry;

pub fn read_directory(backend_id: &'static str, dir: &Path) -> Result<Vec<FileEntry>, String> {
    let entries = fs::read_dir(dir).map_err(|err| err.to_string())?;

    let mut items = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|err| err.to_string())?;
        let metadata = entry.metadata().map_err(|err| err.to_string())?;
        let file_type = entry.file_type().map_err(|err| err.to_string())?;

        items.push(FileEntry {
            name: entry.file_name().to_string_lossy().into_owned(),
            path: disk_path(entry.path(), backend_id),
            is_dir: file_type.is_dir(),
            size: if file_type.is_dir() {
                0
            } else {
                metadata.len()
            },
            modified: metadata.modified().ok(),
        });
    }

    items.sort_by(|left, right| match (left.is_dir, right.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => left.name.to_lowercase().cmp(&right.name.to_lowercase()),
    });

    Ok(items)
}
