use explorer_core::filesystem::{PathMetadata, EPath};

use super::LocalBackend;

impl PathMetadata for LocalBackend {
    fn exists(&self, path: &EPath) -> bool {
        path.disk_ref().map(|disk| disk.exists()).unwrap_or(false)
    }
}
