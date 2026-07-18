use super::{BlockDevice, MountedDevice};

/// A mountable filesystem backend (archives, etc.).
///
/// Host folder access is handled separately by [`super::HostBackend`].
pub trait FsBackend: Send + Sync {
    /// Unique identifier for this backend.
    fn id(&self) -> &'static str;

    /// Whether this backend can mount the given block device.
    fn matches(&self, _device: &BlockDevice) -> bool {
        false
    }

    /// Mount a block device and return a filesystem for accessing its contents.
    ///
    /// Paths on the returned device are relative to the archive / volume root.
    fn mount(&self, device: &BlockDevice) -> Result<Box<dyn MountedDevice>, String>;
}
