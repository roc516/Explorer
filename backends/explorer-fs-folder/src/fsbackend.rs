use std::fs;
use std::path::Path;

use explorer_core::filesystem::{EntryKind, HostBackend, Volume};
use explorer_core::FsEntry;

use crate::directory;

impl HostBackend for crate::FolderBackend {
    fn id(&self) -> &'static str {
        crate::ID
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

    fn list(&self, path: &Path) -> Result<Vec<FsEntry>, String> {
        directory::read_directory(crate::ID, path)
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
