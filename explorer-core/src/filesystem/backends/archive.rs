use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::MountSession;
use crate::filesystem::path::EPath;

pub trait ArchiveMount {
    fn open_session(&self, container: &Path) -> Result<Arc<dyn MountSession>, String> {
        let _ = container;
        Err("archive-session-not-supported".to_string())
    }

    fn close_session(&self, _container: &Path) {}

    fn archive_container<'a>(&self, path: &'a EPath) -> Option<&'a Path>;
    fn nested_archive_file(&self, path: &EPath) -> Option<PathBuf>;

    fn extract_for_open(&self, _container: &Path, _inner: &Path) -> Result<PathBuf, String> {
        Err("extract-not-supported".to_string())
    }
}
