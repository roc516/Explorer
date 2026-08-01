use std::path::Path;

use crate::device::BlockDevice;
use super::MountedFs;

/// A mountable filesystem backend (archives, etc.).
///
/// Host folder access uses the same shape via [`HostBackend`] (path instead of block device).
pub trait FsBackend: Send + Sync {
    /// Unique identifier for this backend.
    fn id(&self) -> &'static str;

    /// Whether this backend can mount the given block device.
    fn matches(&self, _device: &BlockDevice) -> bool {
        false
    }

    /// Mount a block device and return a filesystem for listing entries.
    fn mount(&self, device: &BlockDevice) -> Result<Box<dyn MountedFs>, String>;
}

/// Host folder filesystem — same shape as [`FsBackend`], but keyed by path.
pub trait HostBackend: Send + Sync {
    fn id(&self) -> &'static str;

    /// Whether this backend can mount the given host path.
    ///
    /// An empty path mounts the computer roots (volumes / drives).
    fn matches(&self, path: &Path) -> bool;

    /// Mount a host path and return a filesystem for listing entries.
    fn mount(&self, path: &Path) -> Result<Box<dyn MountedFs>, String>;
}
