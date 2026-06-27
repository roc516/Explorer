use std::path::Path;

use crate::filesystem::Volume;

use super::MountedDevice;

pub trait FsBackend: Send + Sync {
    /// Unique identifier for this backend.
    fn id(&self) -> &'static str;

    /// Whether this backend handles the host filesystem.
    fn is_disk_backend(&self) -> bool {
        false
    }

    /// Whether this backend can mount the given path.
    fn matches(&self, _path: &Path) -> bool {
        false
    }

    /// List top-level volumes / roots (drives on Windows, "/" on Unix).
    fn list_roots(&self) -> Vec<Volume> {
        Vec::new()
    }

    /// Mount the given path and return a device for accessing its contents.
    ///
    /// For a disk backend this returns a device that operates on absolute paths.
    /// For an archive backend this returns a device that operates on paths relative
    /// to the archive root.
    fn mount(&self, path: &Path) -> Result<Box<dyn MountedDevice>, String>;
}
