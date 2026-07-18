use std::path::{Path, PathBuf};

use explorer_core::filesystem::{HostBackend, MountedDevice};
use explorer_core::{DirEntry, FsEntry};

use crate::directory;

enum FolderFs {
    /// Computer roots (volumes / drives); listed via `list("")`.
    Roots,
    Dir(PathBuf),
}

impl MountedDevice for FolderFs {
    fn list(&self, name: &str) -> Result<Vec<FsEntry>, String> {
        match self {
            Self::Roots => {
                if !name.is_empty() {
                    return Err("roots-have-no-subpath".to_string());
                }
                Ok(list_volume_entries())
            }
            Self::Dir(root) => {
                let path = if name.is_empty() {
                    root.clone()
                } else {
                    let mut path = root.clone();
                    for part in name.split(['/', '\\']) {
                        if !part.is_empty() {
                            path.push(part);
                        }
                    }
                    path
                };
                directory::read_directory(&path)
            }
        }
    }
}

fn list_volume_entries() -> Vec<FsEntry> {
    #[cfg(windows)]
    {
        (b'A'..=b'Z')
            .filter_map(|letter| {
                let drive = format!("{}:\\", letter as char);
                let path = PathBuf::from(&drive);
                path.exists().then(|| {
                    FsEntry::Dir(DirEntry {
                        name: drive,
                        path,
                    })
                })
            })
            .collect()
    }
    #[cfg(not(windows))]
    {
        vec![FsEntry::Dir(DirEntry {
            name: "/".to_string(),
            path: PathBuf::from("/"),
        })]
    }
}

impl HostBackend for crate::FolderBackend {
    fn id(&self) -> &'static str {
        crate::ID
    }

    fn matches(&self, path: &Path) -> bool {
        path.as_os_str().is_empty() || path.is_dir()
    }

    fn mount(&self, path: &Path) -> Result<Box<dyn MountedDevice>, String> {
        if path.as_os_str().is_empty() {
            return Ok(Box::new(FolderFs::Roots));
        }
        if !path.is_dir() {
            return Err("not-a-directory".to_string());
        }
        Ok(Box::new(FolderFs::Dir(path.to_path_buf())))
    }
}
