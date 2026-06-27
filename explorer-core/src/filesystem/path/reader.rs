use std::io::{Cursor, Read};

use crate::entry::FileEntry;

use super::epath::EPath;
use super::mounter::Mounter;

pub struct Reader;

impl Reader {
    pub fn read_directory(path: &EPath) -> Result<Vec<FileEntry>, String> {
        let device = Mounter::device(path)?;
        device.list(&path.path)
    }

    pub fn read_file<R>(
        path: &EPath,
        f: impl FnOnce(&mut dyn Read, u64) -> Result<R, String>,
    ) -> Result<R, String> {
        if path.is_directory() {
            return Err("not-a-file".to_string());
        }
        let device = Mounter::device(path)?;
        let bytes = device.read(&path.path)?;
        let len = bytes.len() as u64;
        f(&mut Cursor::new(bytes), len)
    }
}
