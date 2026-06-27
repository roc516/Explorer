use std::path::Path;

use crate::entry::FileEntry;

use super::EntryKind;

/// A mounted filesystem device — represents a mount point that can be listed/read.
///
/// For a disk backend, the device wraps the whole filesystem and paths are absolute.
/// For an archive backend (e.g. zip), the device wraps the archive file and paths are relative.
pub trait MountedDevice: Send + Sync {
    /// List directory contents at the given path.
    fn list(&self, path: &Path) -> Result<Vec<FileEntry>, String>;
    /// Read file bytes at the given path.
    fn read(&self, path: &Path) -> Result<Vec<u8>, String>;
    /// Check if the given path exists.
    fn exists(&self, path: &Path) -> bool;
    /// Return the entry kind (file or directory) for the given path.
    fn entry_kind(&self, path: &Path) -> Option<EntryKind>;
}
