use std::path::PathBuf;

use explorer_core::filesystem::{is_mounted_path, ArchiveMount, PathOps};

use super::LocalBackend;

impl ArchiveMount for LocalBackend {
    fn archive_container<'a>(&self, _path: &'a PathOps) -> Option<&'a std::path::Path> {
        None
    }

    fn nested_archive_file(&self, path: &PathOps) -> Option<PathBuf> {
        let disk = path.disk_ref().ok()?;
        if disk.is_file() && is_mounted_path(disk) {
            Some(disk.to_path_buf())
        } else {
            None
        }
    }
}
