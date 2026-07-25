use std::path::PathBuf;

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

    pub fn from_address(input: &str, context: &Self) -> Self {
        Mounter::from_mount_address(input, context)
            .unwrap_or_else(|| Self::local(input.trim()))
    }

    pub fn backend(&self) -> &'static str {
        self.backend
    }

    pub fn root(&self) -> &DeviceId {
        &self.root
    }

    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    pub fn disk_ref(&self) -> Result<&std::path::Path, String> {
        if Mounter::is_mount(self) {
            return Err("not-a-disk-path".to_string());
        }
        Ok(&self.path)
    }

    pub(crate) fn resolve_mount(&self) -> Result<&dyn FsBackend, String> {
        let registry = try_registry().ok_or("fs backends not initialized".to_string())?;
        registry
            .get(self.backend)
            .ok_or_else(|| format!("unknown-backend:{}", self.backend))
    }
}
