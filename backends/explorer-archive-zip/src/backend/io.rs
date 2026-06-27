use explorer_core::filesystem::{FsIo, Mounter, EPath};

use super::ZipBackend;
use crate::session::zip_session;

impl FsIo for ZipBackend {
    fn read_directory(&self, path: &EPath) -> Result<Vec<explorer_core::FileEntry>, String> {
        let (_, inner) = Mounter::mount_ref(path)?;
        zip_session(path)?.read_directory(inner)
    }

    fn read_file_bytes(&self, path: &EPath) -> Result<Vec<u8>, String> {
        let (_, inner) = Mounter::mount_ref(path)?;
        zip_session(path)?.read_bytes(inner)
    }
}
