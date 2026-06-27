mod backend;
mod kinds;

use std::path::Path;
use std::sync::OnceLock;

pub use backend::FsBackend;
pub use kinds::EntryKind;

pub struct FsRegistry {
    backends: Vec<Box<dyn FsBackend>>,
    disk_backend: Option<&'static str>,
}

impl FsRegistry {
    pub fn new() -> Self {
        Self {
            backends: Vec::new(),
            disk_backend: None,
        }
    }

    pub fn register(&mut self, backend: Box<dyn FsBackend>) {
        if backend.is_disk_backend() {
            self.disk_backend = Some(backend.id());
        }
        self.backends.push(backend);
    }

    pub fn get(&self, id: &str) -> Option<&dyn FsBackend> {
        self.backends
            .iter()
            .find(|backend| backend.id() == id)
            .map(|backend| backend.as_ref())
    }

    pub fn disk_backend(&self) -> Option<&dyn FsBackend> {
        self.disk_backend
            .and_then(|id| self.get(id))
    }

    pub fn find_backend(&self, path: &Path) -> Option<&dyn FsBackend> {
        self.backends
            .iter()
            .find(|backend| backend.matches(path))
            .map(|backend| backend.as_ref())
    }
}

static REGISTRY: OnceLock<FsRegistry> = OnceLock::new();

pub fn ensure_backends_registered(build: impl FnOnce(&mut FsRegistry)) {
    let _ = REGISTRY.get_or_init(|| {
        let mut registry = FsRegistry::new();
        build(&mut registry);
        registry
    });
}

pub fn try_registry() -> Option<&'static FsRegistry> {
    REGISTRY.get()
}

pub fn is_mounted_path(path: &Path) -> bool {
    REGISTRY
        .get()
        .and_then(|registry| registry.find_backend(path))
        .is_some()
}

pub(crate) fn list_drives() -> Vec<crate::filesystem::Volume> {
    REGISTRY
        .get()
        .and_then(|registry| registry.disk_backend())
        .map(|backend| backend.list_roots())
        .unwrap_or_default()
}
