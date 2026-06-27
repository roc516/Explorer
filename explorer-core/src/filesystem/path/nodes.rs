use std::path::PathBuf;

use super::epath::EPath;

pub trait Mountable {
    fn to_epath(&self, backend: &'static str) -> EPath;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Disk {
    pub id: String,
    pub name: String,
    pub volumes: Vec<Volume>,
}

impl Disk {
    pub fn new(id: String, name: String) -> Self {
        Disk {
            id,
            name,
            volumes: Vec::new(),
        }
    }

    pub fn with_volumes(mut self, volumes: Vec<Volume>) -> Self {
        self.volumes = volumes;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Volume {
    pub path: PathBuf,
    pub label: String,
}

impl Volume {
    pub fn new(path: PathBuf, label: String) -> Self {
        Volume { path, label }
    }
}

impl Mountable for Volume {
    fn to_epath(&self, backend: &'static str) -> EPath {
        EPath {
            backend,
            root: self.path.clone(),
            inner: PathBuf::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Directory {
    pub path: EPath,
}

impl Directory {
    pub fn new(path: EPath) -> Self {
        Directory { path }
    }

    pub fn from_volume(volume: &Volume, backend: &'static str) -> Self {
        Directory {
            path: volume.to_epath(backend),
        }
    }

    pub fn parent(&self) -> Option<Self> {
        self.path.parent().map(Self::new)
    }
}

impl Mountable for Directory {
    fn to_epath(&self, _backend: &'static str) -> EPath {
        self.path.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArchiveRoot {
    pub container: PathBuf,
    pub backend: &'static str,
}

impl ArchiveRoot {
    pub fn new(container: PathBuf, backend: &'static str) -> Self {
        ArchiveRoot { container, backend }
    }

    pub fn to_epath(&self) -> EPath {
        EPath {
            backend: self.backend,
            root: self.container.clone(),
            inner: PathBuf::new(),
        }
    }
}

impl Mountable for ArchiveRoot {
    fn to_epath(&self, _backend: &'static str) -> EPath {
        self.to_epath()
    }
}
