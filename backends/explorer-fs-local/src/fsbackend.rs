use std::fs;
use std::path::PathBuf;

use explorer_core::filesystem::{EPath, FsBackend, Volume};

use crate::directory;

impl FsBackend for crate::LocalBackend {
    fn id(&self) -> &'static str {
        crate::ID
    }

    fn is_disk_backend(&self) -> bool {
        true
    }

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

    fn exists(&self, path: &EPath) -> bool {
        path.disk_ref().map(|disk| disk.exists()).unwrap_or(false)
    }

    fn list(&self, path: &EPath) -> Result<Vec<explorer_core::FileEntry>, String> {
        directory::read_directory(crate::ID, path)
    }

    fn read(&self, path: &EPath) -> Result<Vec<u8>, String> {
        let disk = path.disk_ref()?;
        fs::read(disk).map_err(|err| err.to_string())
    }
}
