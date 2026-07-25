use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;

use crate::filesystem::{mount_entry_name, Mounter, EPath};

/// Opaque file content source — obtained when the entry is listed / resolved.
pub trait FileBytes: Send + Sync {
    fn read(&self) -> Result<Vec<u8>, String>;
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

    /// List immediate children of this directory.
    pub fn list(&self) -> Result<Vec<FsEntry>, String> {
        self.directory.list()
    }
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

    /// Resolve a file path to a [`FileEntry`] (lists the parent mount dir when needed).
    pub fn resolve(path: &EPath) -> Result<Self, String> {
        if path.is_directory() {
            return Err("not-a-file".to_string());
        }

        if Mounter::is_mount(path) {
            let full = mount_entry_name(path.path());
            if full.is_empty() {
                return Err("not-a-file".to_string());
            }
            let (parent, child) = match full.rsplit_once('/') {
                Some((parent, child)) => (parent.to_string(), child.to_string()),
                None => (String::new(), full),
            };
            let entries = Mounter::list_at(path, &parent)?;
            for entry in entries {
                if let FsEntry::File(file) = entry {
                    if file.name == child {
                        return Ok(file);
                    }
                }
            }
            Err("file-not-found".to_string())
        } else {
            let disk = path.disk_ref()?;
            let metadata = std::fs::metadata(disk).map_err(|err| err.to_string())?;
            if metadata.is_dir() {
                return Err("not-a-file".to_string());
            }
            Ok(Self::new(
                path.file_name(),
                disk.to_path_buf(),
                metadata.len(),
                metadata.modified().ok(),
                host_file_bytes(disk.to_path_buf()),
            ))
        }
    }

    /// Read this file's bytes through its content handle.
    pub fn read(&self) -> Result<Vec<u8>, String> {
        self.content.read()
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
    fn read(&self) -> Result<Vec<u8>, String> {
        std::fs::read(&self.path).map_err(|err| err.to_string())
    }
}

/// Host-path content handle for directory listings.
pub fn host_file_bytes(path: PathBuf) -> Arc<dyn FileBytes> {
    Arc::new(HostFileBytes { path })
}
