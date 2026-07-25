use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Random-access block I/O — the general block device interface.
pub trait BlockIo: Send + Sync {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> io::Result<usize>;
    fn len(&self) -> u64;
}

/// Stable identity for caching and [`crate::filesystem::EPath`] roots.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DeviceId {
    /// Host filesystem path. Empty path = disk path (not a mount root).
    Host(PathBuf),
    /// An entry inside another mounted filesystem (enables nested archives).
    Nested {
        parent: Box<DeviceId>,
        entry: PathBuf,
    },
}

impl DeviceId {
    pub fn display(&self) -> String {
        match self {
            DeviceId::Host(path) => path.display().to_string(),
            DeviceId::Nested { parent, entry } => {
                let parent = parent.display();
                if entry.as_os_str().is_empty() {
                    parent
                } else if parent.is_empty() {
                    entry.display().to_string()
                } else {
                    format!("{}\\{}", parent, entry.display())
                }
            }
        }
    }

    pub fn host_path(&self) -> Option<&Path> {
        match self {
            DeviceId::Host(path) if !path.as_os_str().is_empty() => Some(path.as_path()),
            _ => None,
        }
    }

    pub fn is_host_disk(&self) -> bool {
        matches!(self, DeviceId::Host(path) if path.as_os_str().is_empty())
    }
}

/// A block device that can be mounted by a filesystem backend.
///
/// Combines a stable [`DeviceId`] with random-access [`BlockIo`].
#[derive(Clone)]
pub struct BlockDevice {
    id: DeviceId,
    name: String,
    io: Arc<dyn BlockIo>,
}

impl std::fmt::Debug for BlockDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlockDevice")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("len", &self.len())
            .finish()
    }
}

impl BlockDevice {
    /// Open a host file as a block device.
    pub fn open_host(path: impl Into<PathBuf>) -> Result<Self, String> {
        let path = path.into();
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.display().to_string());
        let io = HostFileIo::open(&path)?;
        Ok(Self {
            id: DeviceId::Host(path),
            name,
            io: Arc::new(io),
        })
    }

    /// Build a block device from in-memory bytes (e.g. a nested archive read via [`crate::entry::FileEntry`]).
    pub fn from_bytes(id: DeviceId, name: String, data: Vec<u8>) -> Self {
        Self {
            id,
            name,
            io: Arc::new(BytesBlockIo { data }),
        }
    }

    pub fn id(&self) -> &DeviceId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn io(&self) -> &Arc<dyn BlockIo> {
        &self.io
    }

    pub fn read_at(&self, offset: u64, buf: &mut [u8]) -> io::Result<usize> {
        self.io.read_at(offset, buf)
    }

    pub fn len(&self) -> u64 {
        self.io.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

struct HostFileIo {
    file: Mutex<File>,
    len: u64,
}

impl HostFileIo {
    fn open(path: &Path) -> Result<Self, String> {
        let mut file = File::open(path).map_err(|err| err.to_string())?;
        let len = file
            .seek(SeekFrom::End(0))
            .map_err(|err| err.to_string())?;
        Ok(Self {
            file: Mutex::new(file),
            len,
        })
    }
}

impl BlockIo for HostFileIo {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> io::Result<usize> {
        let mut file = self
            .file
            .lock()
            .map_err(|_| io::Error::other("host-file-lock-poisoned"))?;
        file.seek(SeekFrom::Start(offset))?;
        file.read(buf)
    }

    fn len(&self) -> u64 {
        self.len
    }
}

struct BytesBlockIo {
    data: Vec<u8>,
}

impl BlockIo for BytesBlockIo {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> io::Result<usize> {
        if offset as usize >= self.data.len() {
            return Ok(0);
        }
        let start = offset as usize;
        let end = (start + buf.len()).min(self.data.len());
        let n = end - start;
        buf[..n].copy_from_slice(&self.data[start..end]);
        Ok(n)
    }

    fn len(&self) -> u64 {
        self.data.len() as u64
    }
}
