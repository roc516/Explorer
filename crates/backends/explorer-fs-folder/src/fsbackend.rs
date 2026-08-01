use std::path::{Path, PathBuf};
use std::sync::Arc;

use explorer_core::filesystem::{HostBackend, MountedFs};
use explorer_core::{DirEntry, Directory, FsEntry};

use crate::directory;

enum FolderFs {
    /// Computer roots (volumes / drives).
    Roots,
    Dir(PathBuf),
}

impl MountedFs for FolderFs {
    fn list(&self) -> Result<Vec<FsEntry>, String> {
        match self {
            Self::Roots => Ok(list_volume_entries()),
            Self::Dir(root) => list_folder(root),
        }
    }
}

struct FolderDir(PathBuf);

impl Directory for FolderDir {
    fn list(&self) -> Result<Vec<FsEntry>, String> {
        list_folder(&self.0)
    }
}

fn list_folder(root: &Path) -> Result<Vec<FsEntry>, String> {
    directory::read_directory(root, |name, path| {
        DirEntry::new(name, path.clone(), Arc::new(FolderDir(path)))
    })
}

fn list_volume_entries() -> Vec<FsEntry> {
    #[cfg(windows)]
    {
        (b'A'..=b'Z')
            .filter_map(|letter| {
                let drive = format!("{}:\\", letter as char);
                let path = PathBuf::from(&drive);
                path.exists().then(|| {
                    FsEntry::Dir(DirEntry::new(
                        drive,
                        path.clone(),
                        Arc::new(FolderDir(path)),
                    ))
                })
            })
            .collect()
    }
    #[cfg(not(windows))]
    {
        let path = PathBuf::from("/");
        vec![FsEntry::Dir(DirEntry::new(
            "/".to_string(),
            path.clone(),
            Arc::new(FolderDir(path)),
        ))]
    }
}

impl HostBackend for crate::FolderBackend {
    fn id(&self) -> &'static str {
        crate::ID
    }

    fn matches(&self, path: &Path) -> bool {
        path.as_os_str().is_empty() || path.is_dir()
    }

    fn mount(&self, path: &Path) -> Result<Box<dyn MountedFs>, String> {
        if path.as_os_str().is_empty() {
            return Ok(Box::new(FolderFs::Roots));
        }
        if !path.is_dir() {
            return Err("not-a-directory".to_string());
        }
        Ok(Box::new(FolderFs::Dir(path.to_path_buf())))
    }
}
