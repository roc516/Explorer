use std::path::{Path, PathBuf};

use explorer_core::filesystem::{ArchiveMount, PathOps};

use super::ZipBackend;

impl ArchiveMount for ZipBackend {
    fn archive_container<'a>(&self, path: &'a PathOps) -> Option<&'a Path> {
        path.mount_ref().ok().map(|(container, _)| container)
    }

    fn nested_archive_file(&self, _path: &PathOps) -> Option<PathBuf> {
        None
    }

    fn extract_for_open(&self, container: &Path, inner: &Path) -> Result<PathBuf, String> {
        ZipBackend::extract_for_open(self, container, inner)
    }
}