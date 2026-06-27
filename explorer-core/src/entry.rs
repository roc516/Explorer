use std::time::SystemTime;

use crate::filesystem::EPath;

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: EPath,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<SystemTime>,
}
