use std::path::Path;

use crate::entry::FsEntry;
use crate::filesystem::Volume;

use super::EntryKind;

/// Host folder filesystem — calls OS APIs directly, not block-device mounting.
pub trait HostBackend: Send + Sync {
    fn id(&self) -> &'static str;
    fn list_roots(&self) -> Vec<Volume>;
    fn list(&self, path: &Path) -> Result<Vec<FsEntry>, String>;
    fn read(&self, path: &Path) -> Result<Vec<u8>, String>;
    fn exists(&self, path: &Path) -> bool;
    fn entry_kind(&self, path: &Path) -> Option<EntryKind>;
}
