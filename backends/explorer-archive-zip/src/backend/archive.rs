use std::path::Path;
use std::sync::Arc;

use explorer_core::filesystem::{ArchiveMount, MountSession};

use super::ZipBackend;
use crate::session::ZipMountSession;

impl ArchiveMount for ZipBackend {
    fn open_session(&self, container: &Path) -> Result<Arc<dyn MountSession>, String> {
        ZipMountSession::open(container.to_path_buf()).map(|session| session as Arc<dyn MountSession>)
    }
}
