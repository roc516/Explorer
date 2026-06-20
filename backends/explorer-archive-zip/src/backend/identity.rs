use std::path::Path;

use explorer_core::filesystem::BackendIdentity;

use super::ZipBackend;

pub const ID: &str = "zip";
pub const EXTENSIONS: &[&str] = &["zip", "jar", "apk"];

impl BackendIdentity for ZipBackend {
    fn id(&self) -> &'static str {
        ID
    }

    fn matches(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()))
            .unwrap_or(false)
    }
}
