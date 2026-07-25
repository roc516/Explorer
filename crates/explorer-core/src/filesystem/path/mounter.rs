use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use crate::entry::{mount_root_dir, DirEntry, FsEntry};
use crate::filesystem::backends::{try_registry, BlockDevice, DeviceId, MountedDevice};

use super::epath::EPath;
use super::util::mount_entry_name;

type DeviceKey = (&'static str, DeviceId);

static DEVICES: OnceLock<Mutex<HashMap<DeviceKey, Arc<dyn MountedDevice>>>> = OnceLock::new();

fn devices() -> &'static Mutex<HashMap<DeviceKey, Arc<dyn MountedDevice>>> {
    DEVICES.get_or_init(|| Mutex::new(HashMap::new()))
}

pub struct Mounter;

impl Mounter {
    pub(crate) fn mount_path(root: DeviceId, path: PathBuf, backend: &'static str) -> EPath {
        EPath {
            backend,
            root,
            path,
        }
    }

    /// Mount a block device via a mountable backend and return the archive root path.
    pub(crate) fn mount_root(device: BlockDevice) -> Result<EPath, String> {
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

    /// Mount a block device and return the root [`EPath`] plus a listable [`DirEntry`].
    pub fn mount_root_dir(device: BlockDevice) -> Result<(EPath, DirEntry), String> {
        let name = {
            let name = device.name().to_string();
            if name.is_empty() {
                device.id().display()
            } else {
                name
            }
        };
        let root = Self::mount_root(device)?;
        let mounted = Self::device(&root)?;
        Ok((root, mount_root_dir(name, mounted)))
    }

    /// Resolve a navigation path under a mount root to a [`DirEntry`].
    pub fn dir_at(root: &EPath, relative: &Path) -> Result<DirEntry, String> {
        if !Self::is_mount(root) {
            return Err("not-a-mount-path".to_string());
        }
        let name = mount_entry_name(relative);
        if name.is_empty() {
            let mounted = Self::device(root)?;
            return Ok(mount_root_dir(String::new(), mounted));
        }
        Self::dir_entry_at(root, &name)
    }

    /// Cached mounted filesystem root for an archive path.
    pub(crate) fn device(path: &EPath) -> Result<Arc<dyn MountedDevice>, String> {
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

    /// List children at `relative` under the archive root (`""` = mount root).
    ///
    /// Root uses [`MountedDevice::list`]; nested paths walk [`DirEntry`] and call
    /// [`DirEntry::list`].
    pub(crate) fn list_at(path: &EPath, relative: &str) -> Result<Vec<FsEntry>, String> {
        let relative = relative.trim_matches(|c| c == '/' || c == '\\');
        if relative.is_empty() {
            return Self::device(path)?.list();
        }
        Self::dir_entry_at(path, relative)?.list()
    }

    /// Resolve a nested directory entry under the archive root.
    fn dir_entry_at(path: &EPath, relative: &str) -> Result<DirEntry, String> {
        let parts: Vec<&str> = relative
            .split(['/', '\\'])
            .filter(|part| !part.is_empty())
            .collect();
        if parts.is_empty() {
            return Err("empty-directory-path".to_string());
        }

        let mut entries = Self::device(path)?.list()?;
        for (index, part) in parts.iter().enumerate() {
            let dir = entries
                .into_iter()
                .find_map(|entry| match entry {
                    FsEntry::Dir(dir) if dir.name == *part => Some(dir),
                    _ => None,
                })
                .ok_or_else(|| "directory-not-found".to_string())?;
            if index + 1 == parts.len() {
                return Ok(dir);
            }
            entries = dir.list()?;
        }
        Err("directory-not-found".to_string())
    }

    /// Reconstruct a [`BlockDevice`] from a [`DeviceId`] (mountable devices only).
    fn block_device_for(id: &DeviceId) -> Result<BlockDevice, String> {
        match id {
            DeviceId::Host(path) if path.as_os_str().is_empty() => {
                Err("host-disk-is-not-a-block-device".to_string())
            }
            DeviceId::Host(path) => BlockDevice::open_host(path.clone()),
            DeviceId::Nested { parent, entry } => {
                let full = path_to_mount_name(entry);
                let (dir, child) = match full.rsplit_once('/') {
                    Some((dir, child)) => (dir.to_string(), child.to_string()),
                    None => (String::new(), full),
                };
                let parent_root = Self::device_by_id(parent)?;
                let entries = list_under_device(&parent_root, &dir)?;
                let file = entries
                    .into_iter()
                    .find_map(|entry| match entry {
                        FsEntry::File(file) if file.name == child => Some(file),
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

    /// True when this path is inside a mounted archive (not host folder).
    pub fn is_mount(path: &EPath) -> bool {
        !path.root.is_host_disk()
    }

    /// Resolve an internal path inside the same mount as `context`.
    ///
    /// Optionally strips the current container display prefix so pasting a full
    /// `display()` string still works. Never changes root or backend.
    pub(crate) fn from_internal_address(input: &str, context: &EPath) -> Option<EPath> {
        let (container, _) = Self::mount_ref(context).ok()?;
        let container_display = container.display();
        let prefix = format!("{}\\", container_display);
        let inner = input
            .strip_prefix(&prefix)
            .or_else(|| input.strip_prefix(&container_display))
            .unwrap_or(input);
        Some(Self::mount_path(
            context.root.clone(),
            normalize_mount_path(inner),
            context.backend,
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

fn list_under_device(
    root: &Arc<dyn MountedDevice>,
    relative: &str,
) -> Result<Vec<FsEntry>, String> {
    let relative = relative.trim_matches(|c| c == '/' || c == '\\');
    if relative.is_empty() {
        return root.list();
    }

    let parts: Vec<&str> = relative
        .split(['/', '\\'])
        .filter(|part| !part.is_empty())
        .collect();
    let mut entries = root.list()?;
    for (index, part) in parts.iter().enumerate() {
        let dir = entries
            .into_iter()
            .find_map(|entry| match entry {
                FsEntry::Dir(dir) if dir.name == *part => Some(dir),
                _ => None,
            })
            .ok_or_else(|| "directory-not-found".to_string())?;
        if index + 1 == parts.len() {
            return dir.list();
        }
        entries = dir.list()?;
    }
    Err("directory-not-found".to_string())
}
