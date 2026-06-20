use std::path::PathBuf;

use crate::filesystem::backends::FsBackend;

use super::builders::PathBreadcrumb;
use super::ops::EPath;

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
        with_backend_or_false(self, |backend| backend.is_file(self))
    }

    pub fn is_directory(&self) -> bool {
        with_backend_or_false(self, |backend| backend.is_directory(self))
    }

    pub fn file_name(&self) -> String {
        with_backend_or(self, |backend| backend.file_name(self))
    }

    pub fn extension(&self) -> Option<String> {
        self.resolve()
            .ok()
            .and_then(|backend| backend.extension(self))
    }

    pub fn preview_path(&self) -> PathBuf {
        with_backend_or(self, |backend| backend.preview_path(self))
    }

    pub(crate) fn breadcrumbs(&self) -> Vec<PathBreadcrumb> {
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
