use std::path::Path;

use super::MountedDevice;

/// Host folder filesystem — same shape as [`super::FsBackend`], but keyed by path.
pub trait HostBackend: Send + Sync {
    fn id(&self) -> &'static str;

    /// Whether this backend can mount the given host path.
    ///
    /// An empty path mounts the computer roots (volumes / drives).
    fn matches(&self, path: &Path) -> bool;

    /// Mount a host path and return a filesystem for listing entries.
    fn mount(&self, path: &Path) -> Result<Box<dyn MountedDevice>, String>;
}
