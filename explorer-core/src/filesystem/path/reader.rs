use std::io::{Cursor, Read};

use crate::entry::FileEntry;

use super::ops::EPath;

pub struct Reader;

impl Reader {
    pub fn read_directory(path: &EPath) -> Result<Vec<FileEntry>, String> {
        path.resolve()?.read_directory(path)
    }

    pub fn read_file<R>(
        path: &EPath,
        f: impl FnOnce(&mut dyn Read, u64) -> Result<R, String>,
    ) -> Result<R, String> {
        if path.is_directory() {
            return Err("not-a-file".to_string());
        }
        let bytes = path.resolve()?.read_file_bytes(path)?;
        let len = bytes.len() as u64;
        f(&mut Cursor::new(bytes), len)
    }
}
