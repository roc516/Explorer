use std::fs;
use std::path::Path;

use explorer_core::filesystem::{EntryKind, FsBackend, MountedDevice, Volume};

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
                    let path = std::path::PathBuf::from(&drive);
                    path.exists().then_some(Volume::new(path, drive))
                })
                .collect()
        }
        #[cfg(not(windows))]
        {
            vec![Volume::new(std::path::PathBuf::from("/"), "/".to_string())]
        }
    }

    fn mount(&self, _path: &Path) -> Result<Box<dyn MountedDevice>, String> {
        Ok(Box::new(LocalFs { backend_id: crate::ID }))
    }
}

struct LocalFs {
    backend_id: &'static str,
}

impl MountedDevice for LocalFs {
    fn list(&self, path: &Path) -> Result<Vec<explorer_core::FileEntry>, String> {
        directory::read_directory(self.backend_id, path)
    }

    fn read(&self, path: &Path) -> Result<Vec<u8>, String> {
        fs::read(path).map_err(|err| err.to_string())
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn entry_kind(&self, path: &Path) -> Option<EntryKind> {
        if path.is_file() {
            Some(EntryKind::File)
        } else if path.is_dir() {
            Some(EntryKind::Directory)
        } else {
            None
        }
    }
}
