use std::path::{Path, PathBuf};

use crate::filesystem::path::PathOps;

pub trait ArchiveMount {
    fn archive_container<'a>(&self, path: &'a PathOps) -> Option<&'a Path>;
    fn nested_archive_file(&self, path: &PathOps) -> Option<PathBuf>;

    fn extract_for_open(&self, _container: &Path, _inner: &Path) -> Result<PathBuf, String> {
        Err("extract-not-supported".to_string())
    }
}
