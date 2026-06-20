use std::path::Path;

use crate::entry::FileEntry;
use crate::filesystem::path::EPath;

pub trait FsIo {
    fn read_directory(&self, path: &EPath) -> Result<Vec<FileEntry>, String>;
    fn read_file_bytes(&self, path: &EPath) -> Result<Vec<u8>, String>;
    fn system_open_path(&self, path: &EPath) -> Result<std::path::PathBuf, String>;
    fn open_with_system(&self, path: &Path) -> Result<(), String>;
}
