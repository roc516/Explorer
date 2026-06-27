use std::path::PathBuf;

use explorer_core::filesystem::{BackendBootstrap, Volume};

use super::LocalBackend;

impl BackendBootstrap for LocalBackend {
    fn list_roots(&self) -> Vec<Volume> {
        #[cfg(windows)]
        {
            (b'A'..=b'Z')
                .filter_map(|letter| {
                    let drive = format!("{}:\\", letter as char);
                    let path = PathBuf::from(&drive);
                    path.exists().then_some(Volume::new(path, drive))
                })
                .collect()
        }
        #[cfg(not(windows))]
        {
            vec![Volume::new(PathBuf::from("/"), "/".to_string())]
        }
    }

    fn default_initial_path(&self) -> Option<PathBuf> {
        Some(
            dirs::document_dir()
                .or_else(dirs::home_dir)
                .unwrap_or_else(|| PathBuf::from("C:\\")),
        )
    }
}
