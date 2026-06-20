use std::path::Path;

use explorer_core::filesystem::{FsIo, Mounter, EPath};

use super::ZipBackend;

impl FsIo for ZipBackend {
    fn read_directory(&self, path: &EPath) -> Result<Vec<explorer_core::FileEntry>, String> {
        let (container, inner) = Mounter::mount_ref(path)?;
        self.read_zip_directory(container, inner)
    }

    fn read_file_bytes(&self, path: &EPath) -> Result<Vec<u8>, String> {
        let (container, inner) = Mounter::mount_ref(path)?;
        self.read_zip_bytes(container, inner)
    }

    fn system_open_path(&self, path: &EPath) -> Result<std::path::PathBuf, String> {
        let (container, inner) = Mounter::mount_ref(path)?;
        self.extract_for_open(container, inner)
    }

    fn open_with_system(&self, path: &Path) -> Result<(), String> {
        open::that(path).map_err(|err| err.to_string())
    }
}
