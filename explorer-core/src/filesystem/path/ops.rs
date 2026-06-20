use std::io::{Cursor, Read};
use std::path::PathBuf;

use crate::entry::FileEntry;
use crate::filesystem::backends::{try_registry, FsBackend};

use super::builders::PathBreadcrumb;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PathOps {
    pub(crate) backend: &'static str,
    pub(crate) root: PathBuf,
    pub(crate) inner: PathBuf,
}

pub fn disk_path(path: PathBuf, backend: &'static str) -> PathOps {
    PathOps {
        backend,
        root: path,
        inner: PathBuf::new(),
    }
}

pub fn mount_path(container: PathBuf, inner: PathBuf, backend: &'static str) -> PathOps {
    PathOps {
        backend,
        root: container,
        inner,
    }
}

impl PathOps {
    pub fn local(path: impl Into<PathBuf>) -> Self {
        let backend = try_registry()
            .and_then(|registry| registry.disk_backend())
            .expect("disk backend not registered");
        disk_path(path.into(), backend.id())
    }

    pub fn mount_root(container: PathBuf) -> Result<Self, String> {
        let backend = try_registry()
            .ok_or("fs backends not initialized")?
            .find_backend(&container)
            .ok_or("unsupported-archive")?;
        Ok(mount_path(container, PathBuf::new(), backend.id()))
    }

    pub fn disk_ref(&self) -> Result<&std::path::Path, String> {
        if self.is_mount() {
            return Err("not-a-disk-path".to_string());
        }
        Ok(&self.root)
    }

    pub fn mount_ref(&self) -> Result<(&std::path::Path, &std::path::Path), String> {
        if !self.is_mount() {
            return Err("not-a-mount-path".to_string());
        }
        Ok((&self.root, &self.inner))
    }

    pub fn parent(&self) -> Option<PathOps> {
        self.resolve()
            .ok()
            .and_then(|backend| backend.parent(self))
    }

    pub fn display(&self) -> String {
        self.with_backend_or(|backend| backend.display(self))
    }

    pub fn exists(&self) -> bool {
        self.with_backend_or_false(|backend| backend.exists(self))
    }

    pub fn is_file(&self) -> bool {
        self.with_backend_or_false(|backend| backend.is_file(self))
    }

    pub fn is_directory(&self) -> bool {
        self.with_backend_or_false(|backend| backend.is_directory(self))
    }

    pub fn file_name(&self) -> String {
        self.with_backend_or(|backend| backend.file_name(self))
    }

    pub fn extension(&self) -> Option<String> {
        self.resolve()
            .ok()
            .and_then(|backend| backend.extension(self))
    }

    pub fn preview_path(&self) -> PathBuf {
        self.with_backend_or(|backend| backend.preview_path(self))
    }

    pub(crate) fn breadcrumbs(&self) -> Vec<PathBreadcrumb> {
        self.with_backend_or(|backend| backend.breadcrumbs(self))
    }

    pub(crate) fn read_directory(&self) -> Result<Vec<FileEntry>, String> {
        self.resolve()?.read_directory(self)
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

    pub(crate) fn mount_backend(&self) -> Option<&'static str> {
        self.is_mount().then_some(self.backend)
    }

    pub fn open_with_system(&self) -> Result<(), String> {
        let backend = self.resolve()?;
        let path = backend.system_open_path(self)?;
        backend.open_with_system(&path)
    }

    pub fn read_file<R>(
        &self,
        f: impl FnOnce(&mut dyn Read, u64) -> Result<R, String>,
    ) -> Result<R, String> {
        if self.is_directory() {
            return Err("not-a-file".to_string());
        }
        let bytes = self.resolve()?.read_file_bytes(self)?;
        let len = bytes.len() as u64;
        f(&mut Cursor::new(bytes), len)
    }

    pub(crate) fn resolve(&self) -> Result<&dyn FsBackend, String> {
        let registry = try_registry().ok_or("fs backends not initialized".to_string())?;
        registry
            .get(self.backend)
            .ok_or_else(|| format!("unknown-backend:{}", self.backend))
    }

    fn with_backend_or<T: Default>(&self, f: impl FnOnce(&dyn FsBackend) -> T) -> T {
        self.resolve().map(|backend| f(backend)).unwrap_or_default()
    }

    fn with_backend_or_false(&self, f: impl FnOnce(&dyn FsBackend) -> bool) -> bool {
        self.resolve().map(|backend| f(backend)).unwrap_or(false)
    }

    fn is_mount(&self) -> bool {
        self.resolve()
            .map(|backend| !backend.is_disk_backend())
            .unwrap_or(false)
    }
}
