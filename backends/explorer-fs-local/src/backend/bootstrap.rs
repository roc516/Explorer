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
}
