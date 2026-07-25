use crate::entry::FsEntry;

/// A mounted filesystem root — lists only its immediate children.
///
/// Nested directories are listed via [`crate::entry::DirEntry::list`], not this trait.
pub trait MountedDevice: Send + Sync {
    /// List immediate children of the mount root.
    fn list(&self) -> Result<Vec<FsEntry>, String>;
}
