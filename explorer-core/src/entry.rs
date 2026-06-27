use std::time::SystemTime;

use crate::filesystem::EPath;

#[derive(Debug, Clone)]
pub struct DirEntry {
    pub name: String,
    pub path: EPath,
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: EPath,
    pub size: u64,
    pub modified: Option<SystemTime>,
}

#[derive(Debug, Clone)]
pub enum FsEntry {
    Dir(DirEntry),
    File(FileEntry),
}
