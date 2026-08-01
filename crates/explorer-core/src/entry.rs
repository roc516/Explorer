use std::io::{Read, Seek};
use std::path::Path;
use std::sync::Arc;
use std::time::SystemTime;

/// Reader that supports both sequential reads and seeking.
pub trait SeekRead: Read + Seek {}

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
