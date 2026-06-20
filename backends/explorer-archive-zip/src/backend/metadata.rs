use std::path::{Path, PathBuf};

use explorer_core::filesystem::{EntryKind, PathMetadata, PathOps};

use super::ZipBackend;

impl PathMetadata for ZipBackend {
    fn exists(&self, path: &PathOps) -> bool {
        let (container, inner) = match path.mount_ref() {
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

    fn is_file(&self, path: &PathOps) -> bool {
        let Ok((container, inner)) = path.mount_ref() else {
            return false;
        };
        matches!(
            PathMetadata::entry_kind(self, container, inner),
            Some(EntryKind::File)
        )
    }

    fn is_directory(&self, path: &PathOps) -> bool {
        let Ok((container, inner)) = path.mount_ref() else {
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

    fn file_name(&self, path: &PathOps) -> String {
        path.mount_ref()
            .ok()
            .and_then(|(_, inner)| {
                inner
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
            })
            .unwrap_or_default()
    }

    fn extension(&self, path: &PathOps) -> Option<String> {
        path.mount_ref()
            .ok()
            .and_then(|(_, inner)| {
                inner
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(str::to_ascii_lowercase)
            })
    }

    fn preview_path(&self, path: &PathOps) -> PathBuf {
        let (container, inner) = path.mount_ref().unwrap_or((Path::new(""), Path::new("")));
        container.join(inner)
    }

    fn entry_kind(&self, container: &Path, inner: &Path) -> Option<EntryKind> {
        ZipBackend::entry_kind(self, container, inner)
    }
}
