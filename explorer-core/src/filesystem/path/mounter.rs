use std::collections::HashMap;
use std::path::{Component, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use crate::filesystem::backends::{try_registry, MountedDevice};

use super::epath::EPath;

type DeviceKey = (&'static str, PathBuf);

static DEVICES: OnceLock<Mutex<HashMap<DeviceKey, Arc<dyn MountedDevice>>>> = OnceLock::new();

fn devices() -> &'static Mutex<HashMap<DeviceKey, Arc<dyn MountedDevice>>> {
    DEVICES.get_or_init(|| Mutex::new(HashMap::new()))
}

pub struct Mounter;

impl Mounter {
    pub fn mount_path(container: PathBuf, path: PathBuf, backend: &'static str) -> EPath {
        EPath {
            backend,
            root: container,
            path,
        }
    }

    pub fn join_mounted_path(inner: &std::path::Path, name: &str) -> PathBuf {
        if inner.as_os_str().is_empty() {
            PathBuf::from(name)
        } else {
            inner.join(name)
        }
    }

    pub fn mount_root(container: PathBuf) -> Result<EPath, String> {
        let backend = try_registry()
            .ok_or("fs backends not initialized")?
            .find_backend(&container)
            .ok_or("unsupported-archive")?;
        Ok(Self::mount_path(container, PathBuf::new(), backend.id()))
    }

    /// Get or create a mounted device for the given path's container.
    ///
    /// For disk paths this returns a cached disk device.
    /// For mount paths this returns a device bound to the archive container.
    pub fn device(path: &EPath) -> Result<Arc<dyn MountedDevice>, String> {
        let backend = path.resolve()?;

        // For disk paths, cache a single device per backend (stateless)
        if backend.is_disk_backend() {
            let key = (path.backend, PathBuf::new());
            let mut guard = devices().lock().expect("devices poisoned");
            if let Some(device) = guard.get(&key) {
                return Ok(device.clone());
            }
            let device = backend.mount(std::path::Path::new(""))?;
            let device: Arc<dyn MountedDevice> = Arc::from(device);
            guard.insert(key, device.clone());
            return Ok(device);
        }

        // For mount paths, cache per container
        if path.root.as_os_str().is_empty() {
            return backend.mount(std::path::Path::new("")).map(|d| Arc::from(d));
        }
        let key = (path.backend, path.root.clone());
        let mut guard = devices().lock().expect("devices poisoned");
        if let Some(device) = guard.get(&key) {
            return Ok(device.clone());
        }
        let device = backend.mount(&path.root)?;
        let device: Arc<dyn MountedDevice> = Arc::from(device);
        guard.insert(key, device.clone());
        Ok(device)
    }

    pub fn mount_ref(path: &EPath) -> Result<(&std::path::Path, &std::path::Path), String> {
        if !Self::is_mount(path) {
            return Err("not-a-mount-path".to_string());
        }
        Ok((&path.root, &path.path))
    }

    pub(crate) fn mount_backend(path: &EPath) -> Option<&'static str> {
        Self::is_mount(path).then_some(path.backend)
    }

    pub fn is_mount(path: &EPath) -> bool {
        path.resolve()
            .map(|backend| !backend.is_disk_backend())
            .unwrap_or(false)
    }

    pub(crate) fn from_mount_address(input: &str, context: &EPath) -> Option<EPath> {
        let container = context.archive_container()?;
        let trimmed = input.trim();
        let prefix = format!("{}\\", container.display());
        let inner = trimmed
            .strip_prefix(&prefix)
            .or_else(|| trimmed.strip_prefix(&container.display().to_string()))
            .unwrap_or(trimmed);
        let backend = Self::mount_backend(context).or_else(|| {
            try_registry()
                .and_then(|registry| registry.find_backend(container))
                .map(|backend| backend.id())
        })?;
        Some(Self::mount_path(
            container.to_path_buf(),
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
