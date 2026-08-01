use std::path::{Component, PathBuf};
use std::sync::Arc;

use crate::entry::{mount_root_dir, DirEntry};
use crate::filesystem::backends::{try_registry, BlockDevice, MountedDevice};

/// A successfully mounted archive: filesystem handle and root directory.
#[derive(Clone)]
pub struct MountedRoot {
    pub device: Arc<dyn MountedDevice>,
    pub dir: DirEntry,
}

pub struct Mounter;

impl Mounter {
    /// Mount a block device and return its root handles.
    pub fn mount_root_dir(device: BlockDevice) -> Result<MountedRoot, String> {
        let backend = try_registry()
            .ok_or("fs backends not initialized")?
            .find_backend(&device)
            .ok_or("unsupported-archive")?;

        let mounted = backend.mount(&device)?;
        let mounted: Arc<dyn MountedDevice> = Arc::from(mounted);

        Ok(MountedRoot {
            device: mounted.clone(),
            dir: mount_root_dir("/".to_string(), mounted),
        })
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
