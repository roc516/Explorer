use std::fs;
use std::path::{Path, PathBuf};

use crate::entry::FileEntry;

pub fn read_directory(path: &Path) -> Result<Vec<FileEntry>, String> {
    let entries = fs::read_dir(path).map_err(|err| err.to_string())?;

    let mut items = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|err| err.to_string())?;
        let metadata = entry.metadata().map_err(|err| err.to_string())?;
        let file_type = entry.file_type().map_err(|err| err.to_string())?;

        items.push(FileEntry {
            name: entry.file_name().to_string_lossy().into_owned(),
            path: entry.path(),
            is_dir: file_type.is_dir(),
            size: if file_type.is_dir() { 0 } else { metadata.len() },
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

pub fn list_drives() -> Vec<PathBuf> {
    #[cfg(windows)]
    {
        (b'A'..=b'Z')
            .filter_map(|letter| {
                let drive = format!("{}:\\", letter as char);
                let path = PathBuf::from(&drive);
                if path.exists() {
                    Some(path)
                } else {
                    None
                }
            })
            .collect()
    }

    #[cfg(not(windows))]
    {
        vec![PathBuf::from("/")]
    }
}

pub fn default_initial_path() -> PathBuf {
    dirs::document_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| PathBuf::from("C:\\"))
}

pub fn parent_path(path: &Path) -> Option<PathBuf> {
    path.parent().map(Path::to_path_buf)
}

pub fn open_with_system(path: &Path) -> Result<(), String> {
    open::that(path).map_err(|err| err.to_string())
}
