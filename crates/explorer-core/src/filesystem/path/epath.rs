use std::path::{Path, PathBuf};

use crate::filesystem::backends::{try_registry, DeviceId, FsBackend};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EPath {
    pub(crate) backend: &'static str,
    pub(crate) root: DeviceId,
    pub(crate) path: PathBuf,
}

impl EPath {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn resolve_mount(&self) -> Result<&dyn FsBackend, String> {
        let registry = try_registry().ok_or("fs backends not initialized".to_string())?;
        registry
            .get(self.backend)
            .ok_or_else(|| format!("unknown-backend:{}", self.backend))
    }
}
