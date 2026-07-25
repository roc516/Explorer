use std::io::{Cursor, Read};
use std::path::{Component, Path};

use crate::entry::{FileEntry, FsEntry};
use crate::filesystem::backends::host_backend;

use super::epath::EPath;
use super::mounter::Mounter;

pub struct Reader;

impl Reader {
    pub fn read_directory(path: &EPath) -> Result<Vec<FsEntry>, String> {
        if Mounter::is_mount(path) {
            let name = mount_entry_name(&path.path);
            Mounter::list_at(path, &name)
        } else {
            let disk = path.disk_ref()?;
            let host = host_backend();
            if !host.matches(disk) {
                return Err("not-a-directory".to_string());
            }
            host.mount(disk)?.list()
        }
    }

    pub fn read_file<R>(
        entry: &FileEntry,
        f: impl FnOnce(&mut dyn Read, u64) -> Result<R, String>,
    ) -> Result<R, String> {
        let bytes = entry.read()?;
        let len = bytes.len() as u64;
        f(&mut Cursor::new(bytes), len)
    }
}

/// Entry / directory name relative to a mount root (`""`, `"folder"`, `"a/b"`).
pub fn mount_entry_name(path: &Path) -> String {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(name) => Some(name.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}
