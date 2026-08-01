use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use explorer_core::{DirEntry, FileEntry, FsEntry, SeekRead};

pub fn read_directory(
    dir: &Path,
    make_dir: impl Fn(String, PathBuf) -> Arc<dyn DirEntry>,
) -> Result<Vec<FsEntry>, String> {
    let entries = fs::read_dir(dir).map_err(|err| err.to_string())?;

    let mut items = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|err| err.to_string())?;
        let metadata = entry.metadata().map_err(|err| err.to_string())?;
        let file_type = entry.file_type().map_err(|err| err.to_string())?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let disk = entry.path();

        if file_type.is_dir() {
            items.push(FsEntry::Dir(make_dir(name, disk)));
        } else {
            items.push(FsEntry::File(Arc::new(HostFile {
                name,
                path: disk.clone(),
                size: metadata.len(),
                modified: metadata.modified().ok(),
            })));
        }
    }

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

struct HostFile {
    name: String,
    path: PathBuf,
    size: u64,
    modified: Option<SystemTime>,
}

impl FileEntry for HostFile {
    fn name(&self) -> &str {
        &self.name
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn size(&self) -> u64 {
        self.size
    }

    fn modified(&self) -> Option<SystemTime> {
        self.modified
    }

    fn open(&self) -> Result<Box<dyn SeekRead>, String> {
        std::fs::File::open(&self.path)
            .map(|file| Box::new(file) as Box<dyn SeekRead>)
            .map_err(|err| err.to_string())
    }
}
