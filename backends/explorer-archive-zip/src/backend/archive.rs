use std::path::{Path, PathBuf};

use explorer_core::filesystem::{ArchiveMount, Mounter, EPath};

use super::ZipBackend;

impl ArchiveMount for ZipBackend {
    fn archive_container<'a>(&self, path: &'a EPath) -> Option<&'a Path> {
        Mounter::mount_ref(path).ok().map(|(container, _)| container)
    }

    fn nested_archive_file(&self, _path: &EPath) -> Option<PathBuf> {
        None
    }

    fn extract_for_open(&self, container: &Path, inner: &Path) -> Result<PathBuf, String> {
        ZipBackend::extract_for_open(self, container, inner)
    }
}
