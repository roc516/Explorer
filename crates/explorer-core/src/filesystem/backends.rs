mod backend;
mod fs;

use std::sync::OnceLock;

use crate::device::BlockDevice;

pub use backend::{FsBackend, HostBackend};
pub use fs::{entry_at, MountedFs};

/// Registry of mountable backends (archives, etc.). Host FS is separate.
pub struct FsRegistry {
    backends: Vec<Box<dyn FsBackend>>,
}

impl FsRegistry {
    pub fn new() -> Self {
        Self {
            backends: Vec::new(),
        }
    }

    pub fn register(&mut self, backend: Box<dyn FsBackend>) {
        self.backends.push(backend);
    }

    pub fn get(&self, id: &str) -> Option<&dyn FsBackend> {
        self.backends
            .iter()
            .find(|backend| backend.id() == id)
            .map(|backend| backend.as_ref())
    }

    pub fn find_backend(&self, device: &BlockDevice) -> Option<&dyn FsBackend> {
        self.backends
            .iter()
            .find(|backend| backend.matches(device))
            .map(|backend| backend.as_ref())
    }
}

static HOST: OnceLock<Box<dyn HostBackend>> = OnceLock::new();
static REGISTRY: OnceLock<FsRegistry> = OnceLock::new();

/// Register the host folder backend (exactly once).
pub fn ensure_host_registered(host: Box<dyn HostBackend>) {
    let _ = HOST.set(host);
}

/// Register mountable backends (archives, etc.).
pub fn ensure_backends_registered(build: impl FnOnce(&mut FsRegistry)) {
    let _ = REGISTRY.get_or_init(|| {
        let mut registry = FsRegistry::new();
        build(&mut registry);
        registry
    });
}

pub fn try_host() -> Option<&'static dyn HostBackend> {
    HOST.get().map(|h| h.as_ref())
}

pub fn host_backend() -> &'static dyn HostBackend {
    try_host().expect("host backend not registered")
}

pub fn try_registry() -> Option<&'static FsRegistry> {
    REGISTRY.get()
}

/// Whether any registered mountable backend can mount this block device.
pub fn is_mountable(device: &BlockDevice) -> bool {
    REGISTRY
        .get()
        .and_then(|registry| registry.find_backend(device))
        .is_some()
}

pub fn list_drives() -> Vec<std::sync::Arc<dyn crate::entry::DirEntry>> {
    use std::path::Path;

    use crate::entry::FsEntry;

    let Some(host) = try_host() else {
        return Vec::new();
    };
    let roots = Path::new("");
    if !host.matches(roots) {
        return Vec::new();
    }
    let Ok(device) = host.mount(roots) else {
        return Vec::new();
    };
    let Ok(entries) = device.list() else {
        return Vec::new();
    };
    entries
        .into_iter()
        .filter_map(|entry| match entry {
            FsEntry::Dir(dir) => Some(dir),
            FsEntry::File(_) => None,
            FsEntry::Volume(_) => None,
        })
        .collect()
}
