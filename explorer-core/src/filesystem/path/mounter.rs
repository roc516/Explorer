use std::path::{Component, PathBuf};

use crate::filesystem::backends::try_registry;

use super::epath::EPath;
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
