use std::path::PathBuf;

use explorer_core::filesystem::{extension_of, file_name_of, PathMetadata, EPath};

use super::LocalBackend;

impl PathMetadata for LocalBackend {
    fn exists(&self, path: &EPath) -> bool {
        path.disk_ref().map(|disk| disk.exists()).unwrap_or(false)
    }

    fn is_file(&self, path: &EPath) -> bool {
        path.disk_ref().map(|disk| disk.is_file()).unwrap_or(false)
    }

    fn is_directory(&self, path: &EPath) -> bool {
        path.disk_ref().map(|disk| disk.is_dir()).unwrap_or(false)
    }

    fn file_name(&self, path: &EPath) -> String {
        path.disk_ref()
            .map(file_name_of)
            .unwrap_or_default()
    }

    fn extension(&self, path: &EPath) -> Option<String> {
        path.disk_ref().ok().and_then(extension_of)
    }

    fn preview_path(&self, path: &EPath) -> PathBuf {
        path.disk_ref()
            .map(|disk| disk.to_path_buf())
            .unwrap_or_default()
    }
}
