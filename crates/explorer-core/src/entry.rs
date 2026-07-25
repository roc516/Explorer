use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;

use crate::filesystem::backends::{host_backend, MountedDevice};
use crate::filesystem::file_name_of;

/// Opaque file content source — obtained when the entry is listed / resolved.
pub trait FileBytes: Send + Sync {
    /// Open a streaming reader for this file's content.
    fn open(&self) -> Result<Box<dyn Read + Send>, String>;
}

/// Lists immediate children of a directory entry (not a mount root).
pub trait Directory: Send + Sync {
    fn list(&self) -> Result<Vec<FsEntry>, String>;
}

pub struct DirEntry {
    pub name: String,
    pub path: PathBuf,
    directory: Arc<dyn Directory>,
}

impl std::fmt::Debug for DirEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DirEntry")
            .field("name", &self.name)
            .field("path", &self.path)
            .finish_non_exhaustive()
    }
}

impl Clone for DirEntry {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            path: self.path.clone(),
            directory: self.directory.clone(),
        }
    }
}

impl DirEntry {
    pub fn new(name: String, path: PathBuf, directory: Arc<dyn Directory>) -> Self {
        Self {
            name,
            path,
            directory,
        }
    }

    /// Open a host directory as a [`DirEntry`].
    pub fn open_host(path: impl Into<PathBuf>) -> Result<Self, String> {
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
        let mounted: Arc<dyn MountedDevice> = Arc::from(host.mount(&path)?);
        Ok(Self::new(
            name,
            path,
            Arc::new(MountedDirectory(mounted)),
        ))
    }

    /// List immediate children of this directory.
    pub fn list(&self) -> Result<Vec<FsEntry>, String> {
        self.directory.list()
    }
}

struct MountedDirectory(Arc<dyn MountedDevice>);

impl Directory for MountedDirectory {
    fn list(&self) -> Result<Vec<FsEntry>, String> {
        self.0.list()
    }
}

/// Wrap a cached [`MountedDevice`] as a mount-root [`DirEntry`].
pub(crate) fn mount_root_dir(name: String, mounted: Arc<dyn MountedDevice>) -> DirEntry {
    DirEntry::new(
        name,
        PathBuf::new(),
        Arc::new(MountedDirectory(mounted)),
    )
}

#[derive(Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub modified: Option<SystemTime>,
    content: Arc<dyn FileBytes>,
}

impl std::fmt::Debug for FileEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileEntry")
            .field("name", &self.name)
            .field("path", &self.path)
            .field("size", &self.size)
            .field("modified", &self.modified)
            .finish_non_exhaustive()
    }
}

impl FileEntry {
    pub fn new(
        name: String,
        path: PathBuf,
        size: u64,
        modified: Option<SystemTime>,
        content: Arc<dyn FileBytes>,
    ) -> Self {
        Self {
            name,
            path,
            size,
            modified,
            content,
        }
    }

    /// Open a streaming reader for this file's content.
    pub fn open(&self) -> Result<Box<dyn Read + Send>, String> {
        self.content.open()
    }
}

#[derive(Debug, Clone)]
pub enum FsEntry {
    Dir(DirEntry),
    File(FileEntry),
}

struct HostFileBytes {
    path: PathBuf,
}

impl FileBytes for HostFileBytes {
    fn open(&self) -> Result<Box<dyn Read + Send>, String> {
        std::fs::File::open(&self.path)
            .map(|file| Box::new(file) as Box<dyn Read + Send>)
            .map_err(|err| err.to_string())
    }
}

/// Host-path content handle for directory listings.
pub fn host_file_bytes(path: PathBuf) -> Arc<dyn FileBytes> {
    Arc::new(HostFileBytes { path })
}
