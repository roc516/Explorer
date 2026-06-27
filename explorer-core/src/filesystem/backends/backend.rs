use std::path::Path;

use crate::entry::FileEntry;
use crate::filesystem::path::EPath;
use crate::filesystem::Volume;

use super::EntryKind;

pub trait FsBackend: Send + Sync {
    // BackendIdentity
    fn id(&self) -> &'static str;
    fn is_disk_backend(&self) -> bool {
        false
    }
    fn matches(&self, _path: &Path) -> bool {
        false
    }

    // BackendBootstrap
    fn list_roots(&self) -> Vec<Volume> {
        Vec::new()
    }

    // PathMetadata
    fn exists(&self, path: &EPath) -> bool;
    fn kind(&self, _container: &Path, _inner: &Path) -> Option<EntryKind> {
        None
    }

    // FsIo
    fn list(&self, path: &EPath) -> Result<Vec<FileEntry>, String>;
    fn read(&self, path: &EPath) -> Result<Vec<u8>, String>;
}
