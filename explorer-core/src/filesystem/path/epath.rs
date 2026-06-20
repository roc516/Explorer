use std::path::PathBuf;

use crate::filesystem::backends::{try_registry, FsBackend};

use super::mounter::Mounter;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EPath {
    pub(crate) backend: &'static str,
    pub(crate) root: PathBuf,
    pub(crate) inner: PathBuf,
}

pub fn disk_path(path: PathBuf, backend: &'static str) -> EPath {
    EPath {
        backend,
        root: path,
        inner: PathBuf::new(),
    }
}

impl EPath {
    pub fn local(path: impl Into<PathBuf>) -> Self {
        let backend = try_registry()
            .and_then(|registry| registry.disk_backend())
            .expect("disk backend not registered");
        disk_path(path.into(), backend.id())
    }

    pub fn from_address(input: &str, context: &Self) -> Self {
        Mounter::from_mount_address(input, context)
            .unwrap_or_else(|| Self::local(input.trim()))
    }

    pub fn disk_ref(&self) -> Result<&std::path::Path, String> {
        if Mounter::is_mount(self) {
            return Err("not-a-disk-path".to_string());
        }
        Ok(&self.root)
    }

    pub(crate) fn resolve(&self) -> Result<&dyn FsBackend, String> {
        let registry = try_registry().ok_or("fs backends not initialized".to_string())?;
        registry
            .get(self.backend)
            .ok_or_else(|| format!("unknown-backend:{}", self.backend))
    }
}
