use std::path::Path;

use explorer_core::filesystem::{EntryKind, Mounter, PathMetadata, EPath};

use super::ZipBackend;
use crate::session::{zip_session, zip_session_for};

impl PathMetadata for ZipBackend {
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

    fn entry_kind(&self, container: &Path, inner: &Path) -> Option<EntryKind> {
        let session = zip_session_for(container).ok()?;
        if inner.as_os_str().is_empty() {
            return Some(EntryKind::Directory);
        }
        session.entry_kind(inner)
    }
}
