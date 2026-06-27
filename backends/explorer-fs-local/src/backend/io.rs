use std::fs;

use explorer_core::filesystem::{FsIo, EPath};

use super::identity::ID;
use super::LocalBackend;
use crate::directory;

impl FsIo for LocalBackend {
    fn read_directory(&self, path: &EPath) -> Result<Vec<explorer_core::FileEntry>, String> {
        directory::read_directory(ID, path)
    }

    fn read_file_bytes(&self, path: &EPath) -> Result<Vec<u8>, String> {
        let disk = path.disk_ref()?;
        fs::read(disk).map_err(|err| err.to_string())
    }
}
