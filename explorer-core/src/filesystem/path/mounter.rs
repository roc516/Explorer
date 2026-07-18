use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use crate::filesystem::backends::{try_registry, BlockDevice, DeviceId, MountedDevice};

use super::epath::EPath;

type DeviceKey = (&'static str, DeviceId);

static DEVICES: OnceLock<Mutex<HashMap<DeviceKey, Arc<dyn MountedDevice>>>> = OnceLock::new();

fn devices() -> &'static Mutex<HashMap<DeviceKey, Arc<dyn MountedDevice>>> {
    DEVICES.get_or_init(|| Mutex::new(HashMap::new()))
}

pub struct Mounter;

impl Mounter {
    pub fn mount_path(root: DeviceId, path: PathBuf, backend: &'static str) -> EPath {
        EPath {
            backend,
            root,
            path,
        }
    }

    pub fn join_mounted_path(inner: &Path, name: &str) -> PathBuf {
        if inner.as_os_str().is_empty() {
            PathBuf::from(name)
        } else {
            inner.join(name)
        }
    }

    /// Mount a block device via a mountable backend and return the archive root path.
    pub fn mount_root(device: BlockDevice) -> Result<EPath, String> {
        let backend = try_registry()
            .ok_or("fs backends not initialized")?
            .find_backend(&device)
            .ok_or("unsupported-archive")?;
        let backend_id = backend.id();
        let key = (backend_id, device.id().clone());

        let mounted = backend.mount(&device)?;
        let mounted: Arc<dyn MountedDevice> = Arc::from(mounted);
        devices()
            .lock()
            .expect("devices poisoned")
            .insert(key, mounted);

        Ok(Self::mount_path(
            device.id().clone(),
            PathBuf::new(),
            backend_id,
        ))
    }

    /// Cached mounted filesystem for an archive path.
    pub fn device(path: &EPath) -> Result<Arc<dyn MountedDevice>, String> {
        if !Self::is_mount(path) {
            return Err("not-a-mount-path".to_string());
        }

        let backend = path.resolve_mount()?;
        let key = (path.backend, path.root.clone());

        {
            let guard = devices().lock().expect("devices poisoned");
            if let Some(device) = guard.get(&key) {
                return Ok(device.clone());
            }
        }

        let block = Self::block_device_for(&path.root)?;
        let mounted = backend.mount(&block)?;
        let mounted: Arc<dyn MountedDevice> = Arc::from(mounted);
        devices()
            .lock()
            .expect("devices poisoned")
            .insert(key, mounted.clone());
        Ok(mounted)
    }

    /// Reconstruct a [`BlockDevice`] from a [`DeviceId`] (mountable devices only).
    pub fn block_device_for(id: &DeviceId) -> Result<BlockDevice, String> {
        match id {
            DeviceId::Host(path) if path.as_os_str().is_empty() => {
                Err("host-disk-is-not-a-block-device".to_string())
            }
            DeviceId::Host(path) => BlockDevice::open_host(path.clone()),
            DeviceId::Nested { parent, entry } => {
                let parent_fs = Self::device_by_id(parent)?;
                let full = path_to_mount_name(entry);
                let (dir, child) = match full.rsplit_once('/') {
                    Some((dir, child)) => (dir.to_string(), child.to_string()),
                    None => (String::new(), full),
                };
                let file = parent_fs
                    .list(&dir)?
                    .into_iter()
                    .find_map(|entry| match entry {
                        crate::entry::FsEntry::File(file) if file.name == child => Some(file),
                        _ => None,
                    })
                    .ok_or_else(|| "file-not-found".to_string())?;
                let data = file.read()?;
                Ok(BlockDevice::from_bytes(id.clone(), file.name, data))
            }
        }
    }

    fn device_by_id(id: &DeviceId) -> Result<Arc<dyn MountedDevice>, String> {
        {
            let guard = devices().lock().expect("devices poisoned");
            for ((_, cached_id), device) in guard.iter() {
                if cached_id == id {
                    return Ok(device.clone());
                }
            }
        }

        let block = Self::block_device_for(id)?;
        let backend = try_registry()
            .ok_or("fs backends not initialized")?
            .find_backend(&block)
            .ok_or("unsupported-archive")?;
        let key = (backend.id(), id.clone());
        let mounted = backend.mount(&block)?;
        let mounted: Arc<dyn MountedDevice> = Arc::from(mounted);
        devices()
            .lock()
            .expect("devices poisoned")
            .insert(key, mounted.clone());
        Ok(mounted)
    }

    pub fn mount_ref(path: &EPath) -> Result<(&DeviceId, &Path), String> {
        if !Self::is_mount(path) {
            return Err("not-a-mount-path".to_string());
        }
        Ok((&path.root, &path.path))
    }

    pub(crate) fn mount_backend(path: &EPath) -> Option<&'static str> {
        Self::is_mount(path).then_some(path.backend)
    }

    /// True when this path is inside a mounted archive (not host folder).
    pub fn is_mount(path: &EPath) -> bool {
        !path.root.is_host_disk()
    }

    pub(crate) fn from_mount_address(input: &str, context: &EPath) -> Option<EPath> {
        let (container, _) = Self::mount_ref(context).ok()?;
        let trimmed = input.trim();
        let prefix = format!("{}\\", container.display());
        let inner = trimmed
            .strip_prefix(&prefix)
            .or_else(|| trimmed.strip_prefix(&container.display()))
            .unwrap_or(trimmed);
        let backend = Self::mount_backend(context).or_else(|| {
            Self::block_device_for(container)
                .ok()
                .and_then(|device| {
                    try_registry()
                        .and_then(|registry| registry.find_backend(&device))
                        .map(|backend| backend.id())
                })
        })?;
        Some(Self::mount_path(
            container.clone(),
            normalize_mount_path(inner),
            backend,
        ))
    }
}

fn normalize_mount_path(value: &str) -> PathBuf {
    let mut result = PathBuf::new();
    for component in std::path::Path::new(value).components() {
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

fn path_to_mount_name(path: &Path) -> String {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(name) => Some(name.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}
