use std::path::Path;

use explorer_core::filesystem::{FsIo, PathOps};

use super::ZipBackend;

impl FsIo for ZipBackend {
    fn read_directory(&self, path: &PathOps) -> Result<Vec<explorer_core::FileEntry>, String> {
        let (container, inner) = path.mount_ref()?;
        self.read_zip_directory(container, inner)
    }

    fn read_file_bytes(&self, path: &PathOps) -> Result<Vec<u8>, String> {
        let (container, inner) = path.mount_ref()?;
        self.read_zip_bytes(container, inner)
    }

    fn system_open_path(&self, path: &PathOps) -> Result<std::path::PathBuf, String> {
        let (container, inner) = path.mount_ref()?;
        self.extract_for_open(container, inner)
    }

    fn open_with_system(&self, path: &Path) -> Result<(), String> {
        open::that(path).map_err(|err| err.to_string())
    }
}
