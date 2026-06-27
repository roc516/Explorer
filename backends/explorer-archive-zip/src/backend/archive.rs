use std::path::{Path, PathBuf};
use std::sync::Arc;

use explorer_core::filesystem::{ArchiveMount, MountSession, Mounter, EPath};

use super::ZipBackend;
use crate::session::ZipMountSession;

impl ArchiveMount for ZipBackend {
    fn open_session(&self, container: &Path) -> Result<Arc<dyn MountSession>, String> {
        ZipMountSession::open(container.to_path_buf()).map(|session| session as Arc<dyn MountSession>)
    }

    fn archive_container<'a>(&self, path: &'a EPath) -> Option<&'a Path> {
        Mounter::mount_ref(path).ok().map(|(container, _)| container)
    }

    fn nested_archive_file(&self, _path: &EPath) -> Option<PathBuf> {
        None
    }

    fn extract_for_open(&self, container: &Path, inner: &Path) -> Result<PathBuf, String> {
        crate::session::zip_session_for(container)?.extract_for_open(inner)
    }
}
