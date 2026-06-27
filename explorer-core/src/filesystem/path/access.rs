use std::path::PathBuf;

use crate::filesystem::backends::FsBackend;

use super::builders::PathBreadcrumb;
use super::epath::EPath;
use super::mounter::Mounter;
use crate::filesystem::EntryKind;

impl EPath {
    pub fn parent(&self) -> Option<EPath> {
        self.resolve()
            .ok()
            .and_then(|backend| backend.parent(self))
    }

    pub fn display(&self) -> String {
        with_backend_or(self, |backend| backend.display(self))
    }

    pub fn exists(&self) -> bool {
        with_backend_or_false(self, |backend| backend.exists(self))
    }

    pub fn is_file(&self) -> bool {
        if let Ok(disk) = self.disk_ref() {
            return disk.is_file();
        }
        let Ok(backend) = self.resolve() else { return false };
        let Ok((container, inner)) = Mounter::mount_ref(self) else { return false };
        matches!(backend.entry_kind(container, inner), Some(EntryKind::File))
    }

    pub fn is_directory(&self) -> bool {
        if let Ok(disk) = self.disk_ref() {
            return disk.is_dir();
        }
        let Ok(backend) = self.resolve() else { return false };
        let Ok((container, inner)) = Mounter::mount_ref(self) else { return false };
        matches!(backend.entry_kind(container, inner), Some(EntryKind::Directory))
    }

    pub fn file_name(&self) -> String {
        self.path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default()
    }

    pub fn extension(&self) -> Option<String> {
        self.path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(str::to_ascii_lowercase)
    }

    pub fn preview_path(&self) -> PathBuf {
        with_backend_or(self, |backend| backend.preview_path(self))
    }

    pub fn breadcrumbs(&self) -> Vec<PathBreadcrumb> {
        with_backend_or(self, |backend| backend.breadcrumbs(self))
    }

    pub(crate) fn archive_container(&self) -> Option<&std::path::Path> {
        self.resolve()
            .ok()
            .and_then(|backend| backend.archive_container(self))
    }

    pub fn nested_archive_file(&self) -> Option<PathBuf> {
        self.resolve()
            .ok()
            .and_then(|backend| backend.nested_archive_file(self))
    }

    pub fn open_with_system(&self) -> Result<(), String> {
        let backend = self.resolve()?;
        let path = backend.system_open_path(self)?;
        backend.open_with_system(&path)
    }
}

fn with_backend_or<T: Default>(path: &EPath, f: impl FnOnce(&dyn FsBackend) -> T) -> T {
    path.resolve().map(|backend| f(backend)).unwrap_or_default()
}

fn with_backend_or_false(path: &EPath, f: impl FnOnce(&dyn FsBackend) -> bool) -> bool {
    path.resolve().map(|backend| f(backend)).unwrap_or(false)
}
