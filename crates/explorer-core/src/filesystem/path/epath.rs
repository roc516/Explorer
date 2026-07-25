use std::path::{Path, PathBuf};

use crate::filesystem::backends::{host_backend, try_registry, DeviceId, FsBackend};

use super::mounter::Mounter;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EPath {
    pub(crate) backend: &'static str,
    pub(crate) root: DeviceId,
    pub(crate) path: PathBuf,
}

pub fn disk_path(disk_path: PathBuf, backend: &'static str) -> EPath {
    EPath {
        backend,
        root: DeviceId::Host(PathBuf::new()),
        path: disk_path,
    }
}

impl EPath {
    pub fn local(path: impl Into<PathBuf>) -> Self {
        disk_path(path.into(), host_backend().id())
    }

    /// Resolve an address-bar input relative to `context` only.
    ///
    /// Mount contexts stay on the same root/backend; disk contexts stay on the
    /// host filesystem. Does not open or switch archives from the address bar.
    pub fn from_address(input: &str, context: &Self) -> Self {
        let trimmed = input.trim();
        if Mounter::is_mount(context) {
            Mounter::from_internal_address(trimmed, context)
                .unwrap_or_else(|| context.clone())
        } else {
            Self::from_disk_address(trimmed, context)
        }
    }

    /// In-window navigation path (disk absolute path, or path inside the mount).
    pub fn navigation_path(&self) -> PathBuf {
        self.path.clone()
    }

    /// Rebuild an [`EPath`] in the same window context with a new navigation path.
    pub fn with_navigation_path(&self, path: PathBuf) -> Self {
        if Mounter::is_mount(self) {
            Mounter::mount_path(self.root.clone(), path, self.backend)
        } else {
            disk_path(path, self.backend)
        }
    }

    /// Path shown while editing the address bar (internal path inside a mount).
    pub fn internal_display(&self) -> String {
        if Mounter::is_mount(self) {
            self.path.display().to_string()
        } else {
            self.display()
        }
    }

    pub fn backend(&self) -> &'static str {
        self.backend
    }

    pub fn root(&self) -> &DeviceId {
        &self.root
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn disk_ref(&self) -> Result<&Path, String> {
        if Mounter::is_mount(self) {
            return Err("not-a-disk-path".to_string());
        }
        Ok(&self.path)
    }

    fn from_disk_address(input: &str, context: &Self) -> Self {
        let path = PathBuf::from(input);
        let resolved = if path.is_absolute() {
            path
        } else {
            context.path.join(path)
        };
        disk_path(resolved, context.backend)
    }

    pub(crate) fn resolve_mount(&self) -> Result<&dyn FsBackend, String> {
        let registry = try_registry().ok_or("fs backends not initialized".to_string())?;
        registry
            .get(self.backend)
            .ok_or_else(|| format!("unknown-backend:{}", self.backend))
    }
}
