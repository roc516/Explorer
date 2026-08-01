use std::sync::{Arc, Mutex};

use explorer_core::BlockDevice;
use explorer_core::filesystem::MountedFs;
use explorer_core::FsEntry;
use zip::ZipArchive;

use crate::dir::read_directory;
use crate::reader::BlockReader;

pub(crate) struct ZipEntryRecord {
    pub(crate) name: String,
    pub(crate) size: u64,
    pub(crate) index: usize,
}

/// Mount root of a zip archive.
pub(crate) struct ZipFs {
    entries: Arc<Vec<ZipEntryRecord>>,
    archive: Arc<Mutex<ZipArchive<BlockReader>>>,
}

impl ZipFs {
    pub(crate) fn open(device: &BlockDevice) -> Result<Self, String> {
        let reader = BlockReader::new(device.io().clone());
        let mut archive = ZipArchive::new(reader).map_err(|err| err.to_string())?;
        let mut entries = Vec::with_capacity(archive.len());

        for index in 0..archive.len() {
            let entry = archive.by_index(index).map_err(|err| err.to_string())?;
            let name = entry.name().replace('\\', "/");
            if name.ends_with('/') {
                continue;
            }
            entries.push(ZipEntryRecord {
                name,
                size: entry.size(),
                index,
            });
        }

        Ok(Self {
            entries: Arc::new(entries),
            archive: Arc::new(Mutex::new(archive)),
        })
    }
}

impl MountedFs for ZipFs {
    fn list(&self) -> Result<Vec<FsEntry>, String> {
        read_directory(&self.entries, &self.archive, "")
    }
}
