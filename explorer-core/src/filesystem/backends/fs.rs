use crate::entry::FsEntry;

/// A mounted filesystem — directory listing only.
///
/// To read file data, obtain a [`crate::entry::FileEntry`] from [`list`](Self::list)
/// (or [`FileEntry::resolve`](crate::entry::FileEntry::resolve)) and call
/// [`FileEntry::read`](crate::entry::FileEntry::read).
pub trait MountedDevice: Send + Sync {
    /// List immediate children of the directory named `name`.
    ///
    /// `name` is relative to the mount root (e.g. `""`, `"folder"`, `"a/b"`).
    /// It is not a host filesystem path.
    fn list(&self, name: &str) -> Result<Vec<FsEntry>, String>;
}
