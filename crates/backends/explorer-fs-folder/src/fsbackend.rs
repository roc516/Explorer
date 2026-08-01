use std::path::{Path, PathBuf};
use std::sync::Arc;

use explorer_core::filesystem::{HostBackend, MountedFs};
use explorer_core::{DirEntry, FsEntry};

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

struct FolderDir {
    name: String,
    path: PathBuf,
}

impl DirEntry for FolderDir {
    fn name(&self) -> &str {
        &self.name
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn list(&self) -> Result<Vec<FsEntry>, String> {
        list_folder(&self.path)
    }
}

fn list_folder(root: &Path) -> Result<Vec<FsEntry>, String> {
    directory::read_directory(root, |name, path| {
        Arc::new(FolderDir { name, path }) as Arc<dyn DirEntry>
    })
}

fn list_volume_entries() -> Vec<FsEntry> {
    let root = if cfg!(windows) {
        PathBuf::from("C:\\")
    } else {
        PathBuf::from("/")
    };
    vec![FsEntry::Dir(
        Arc::new(FolderDir {
            name: root.to_string_lossy().into_owned(),
            path: root,
        }) as Arc<dyn DirEntry>,
    )]
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
