use std::io::{Read, Seek};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use crate::filesystem::backends::{host_backend, MountedFs};
use crate::filesystem::file_name_of;

/// Reader that supports both sequential reads and seeking.
pub trait SeekRead: Read + Seek + Send {}

impl<T: Read + Seek + Send> SeekRead for T {}

/// A directory entry — name, path, and a backend-specific `list`.
pub trait DirEntry: Send + Sync {
    fn name(&self) -> &str;
    fn path(&self) -> &Path;
    fn list(&self) -> Result<Vec<FsEntry>, String>;
}

impl std::fmt::Debug for dyn DirEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DirEntry")
            .field("name", &self.name())
            .field("path", &self.path())
            .finish_non_exhaustive()
    }
}

struct MountedDir {
    name: String,
    path: PathBuf,
    mounted: Arc<dyn MountedFs>,
}

impl DirEntry for MountedDir {
    fn name(&self) -> &str {
        &self.name
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn list(&self) -> Result<Vec<FsEntry>, String> {
        self.mounted.list()
    }
}

/// Open a host directory as a [`DirEntry`].
pub fn open_host_dir(path: impl Into<PathBuf>) -> Result<Arc<dyn DirEntry>, String> {
    let path = path.into();
    let host = host_backend();
    if !host.matches(&path) {
        return Err("not-a-directory".to_string());
    }
    let meta = std::fs::metadata(&path).map_err(|err| err.to_string())?;
    if !meta.is_dir() {
        return Err("not-a-directory".to_string());
    }
    let name = {
        let name = file_name_of(&path);
        if name.is_empty() {
            path.display().to_string()
        } else {
            name
        }
    };
    let mounted: Arc<dyn MountedFs> = Arc::from(host.mount(&path)?);
    Ok(Arc::new(MountedDir { name, path, mounted }))
}

/// A file entry — name, metadata, and a backend-specific `open`.
pub trait FileEntry: Send + Sync {
    fn name(&self) -> &str;
    fn path(&self) -> &Path;
    fn size(&self) -> u64;
    fn modified(&self) -> Option<SystemTime>;
    fn open(&self) -> Result<Box<dyn SeekRead>, String>;
}

impl std::fmt::Debug for dyn FileEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileEntry")
            .field("name", &self.name())
            .field("path", &self.path())
            .field("size", &self.size())
            .field("modified", &self.modified())
            .finish_non_exhaustive()
    }
}

/// A disk volume — name, openable as a byte stream, and listable as a directory.
pub trait VolumeEntry: Send + Sync {
    fn name(&self) -> &str;
    fn open(&self) -> Result<Box<dyn SeekRead>, String>;
    fn list(&self) -> Result<Vec<FsEntry>, String>;
}

impl std::fmt::Debug for dyn VolumeEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VolumeEntry")
            .field("name", &self.name())
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone)]
pub enum FsEntry {
    Dir(Arc<dyn DirEntry>),
    File(Arc<dyn FileEntry>),
    Volume(Arc<dyn VolumeEntry>),
}
