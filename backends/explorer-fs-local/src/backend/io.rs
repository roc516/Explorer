use std::fs;
use std::path::Path;

use explorer_core::filesystem::{FsIo, PathOps};

use super::identity::ID;
use super::LocalBackend;
use crate::directory;

impl FsIo for LocalBackend {
    fn read_directory(&self, path: &PathOps) -> Result<Vec<explorer_core::FileEntry>, String> {
        directory::read_directory(ID, path)
    }

    fn read_file_bytes(&self, path: &PathOps) -> Result<Vec<u8>, String> {
        let disk = path.disk_ref()?;
        fs::read(disk).map_err(|err| err.to_string())
    }

    fn system_open_path(&self, path: &PathOps) -> Result<std::path::PathBuf, String> {
        Ok(path.disk_ref()?.to_path_buf())
    }

    fn open_with_system(&self, path: &Path) -> Result<(), String> {
        open::that(path).map_err(|err| err.to_string())
    }
}
