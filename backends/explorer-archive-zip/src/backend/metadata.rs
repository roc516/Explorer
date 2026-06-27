use std::path::{Path, PathBuf};

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

    fn is_file(&self, path: &EPath) -> bool {
        let Ok((_, inner)) = Mounter::mount_ref(path) else {
            return false;
        };
        matches!(
            zip_session(path)
                .ok()
                .and_then(|session| session.entry_kind(inner)),
            Some(EntryKind::File)
        )
    }

    fn is_directory(&self, path: &EPath) -> bool {
        let Ok((_, inner)) = Mounter::mount_ref(path) else {
            return false;
        };
        if inner.as_os_str().is_empty() {
            return true;
        }
        matches!(
            zip_session(path)
                .ok()
                .and_then(|session| session.entry_kind(inner)),
            Some(EntryKind::Directory)
        )
    }

    fn file_name(&self, path: &EPath) -> String {
        Mounter::mount_ref(path)
            .ok()
            .and_then(|(_, inner)| {
                inner
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
            })
            .unwrap_or_default()
    }

    fn extension(&self, path: &EPath) -> Option<String> {
        Mounter::mount_ref(path)
            .ok()
            .and_then(|(_, inner)| {
                inner
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(str::to_ascii_lowercase)
            })
    }

    fn preview_path(&self, path: &EPath) -> PathBuf {
        let (container, inner) = Mounter::mount_ref(path).unwrap_or((Path::new(""), Path::new("")));
        container.join(inner)
    }

    fn entry_kind(&self, container: &Path, inner: &Path) -> Option<EntryKind> {
        zip_session_for(container)
            .ok()
            .and_then(|session| session.entry_kind(inner))
    }
}
