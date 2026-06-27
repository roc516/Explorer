use std::path::PathBuf;

use explorer_core::filesystem::{PathMetadata, EPath};

use super::LocalBackend;

impl PathMetadata for LocalBackend {
    fn exists(&self, path: &EPath) -> bool {
        path.disk_ref().map(|disk| disk.exists()).unwrap_or(false)
    }

    fn preview_path(&self, path: &EPath) -> PathBuf {
        path.disk_ref()
            .map(|disk| disk.to_path_buf())
            .unwrap_or_default()
    }
}
