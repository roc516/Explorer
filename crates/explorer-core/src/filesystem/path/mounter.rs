use std::path::{Component, PathBuf};
use std::sync::Arc;

use crate::device::BlockDevice;
use crate::filesystem::backends::{try_registry, MountedFs};

pub struct Mounter;

impl Mounter {
    /// Mount a block device and return its filesystem handle.
    pub fn mount_root_dir(device: BlockDevice) -> Result<Arc<dyn MountedFs>, String> {
        let backend = try_registry()
            .ok_or("fs backends not initialized")?
            .find_backend(&device)
            .ok_or("unsupported-archive")?;

        let mounted = backend.mount(&device)?;
        Ok(Arc::from(mounted))
    }

    /// Parse an address-bar string into a mount-internal navigation path.
    pub fn parse_internal_path(input: &str) -> PathBuf {
        let mut result = PathBuf::new();
        for component in std::path::Path::new(input).components() {
            match component {
                Component::Normal(name) => result.push(name),
                Component::ParentDir => {
                    result.pop();
                }
                _ => {}
            }
        }
        result
    }
}
