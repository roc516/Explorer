use std::path::{Path, PathBuf};

use explorer_core::filesystem::{EntryKind, Mounter, PathMetadata, EPath};

use super::ZipBackend;

impl PathMetadata for ZipBackend {
    fn exists(&self, path: &EPath) -> bool {
        let (container, inner) = match Mounter::mount_ref(path) {
            Ok(parts) => parts,
            Err(_) => return false,
        };
        if !container.is_file() {
            return false;
        }
        if inner.as_os_str().is_empty() {
            return true;
        }
        PathMetadata::entry_kind(self, container, inner).is_some()
    }

    fn is_file(&self, path: &EPath) -> bool {
        let Ok((container, inner)) = Mounter::mount_ref(path) else {
            return false;
        };
        matches!(
            PathMetadata::entry_kind(self, container, inner),
            Some(EntryKind::File)
        )
    }

    fn is_directory(&self, path: &EPath) -> bool {
        let Ok((container, inner)) = Mounter::mount_ref(path) else {
            return false;
        };
        if inner.as_os_str().is_empty() {
            return true;
        }
        matches!(
            PathMetadata::entry_kind(self, container, inner),
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
        ZipBackend::entry_kind(self, container, inner)
    }
}
