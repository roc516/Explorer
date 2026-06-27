use std::path::Path;
use std::sync::Arc;

use explorer_core::filesystem::{EntryKind, EPath, FsBackend, MountSession, Mounter};

use super::ZipBackend;
use crate::session::{zip_session, zip_session_for, ZipMountSession};

pub const ID: &str = "zip";
pub const EXTENSIONS: &[&str] = &["zip", "jar", "apk"];

impl FsBackend for ZipBackend {
    fn id(&self) -> &'static str {
        ID
    }

    fn matches(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()))
            .unwrap_or(false)
    }

    fn exists(&self, path: &EPath) -> bool {
        let Ok((container, inner)) = Mounter::mount_ref(path) else {
            return false;
        };
        if !container.is_file() {
            return false;
        }
        if inner.as_os_str().is_empty() {
            return true;
        }
        zip_session(path)
            .ok()
            .and_then(|session| session.entry_kind(inner))
            .is_some()
    }

    fn kind(&self, container: &Path, inner: &Path) -> Option<EntryKind> {
        let session = zip_session_for(container).ok()?;
        if inner.as_os_str().is_empty() {
            return Some(EntryKind::Directory);
        }
        session.entry_kind(inner)
    }

    fn list(&self, path: &EPath) -> Result<Vec<explorer_core::FileEntry>, String> {
        let (_, inner) = Mounter::mount_ref(path)?;
        zip_session(path)?.read_directory(inner)
    }

    fn read(&self, path: &EPath) -> Result<Vec<u8>, String> {
        let (_, inner) = Mounter::mount_ref(path)?;
        zip_session(path)?.read_bytes(inner)
    }

    fn open(&self, container: &Path) -> Result<Arc<dyn MountSession>, String> {
        ZipMountSession::open(container.to_path_buf())
            .map(|session| session as Arc<dyn MountSession>)
    }
}
