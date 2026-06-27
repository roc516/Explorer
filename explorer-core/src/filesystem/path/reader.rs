use std::io::{Cursor, Read};

use crate::entry::FileEntry;

use super::epath::EPath;

pub struct Reader;

impl Reader {
    pub fn read_directory(path: &EPath) -> Result<Vec<FileEntry>, String> {
        path.resolve()?.list(path)
    }

    pub fn read_file<R>(
        path: &EPath,
        f: impl FnOnce(&mut dyn Read, u64) -> Result<R, String>,
    ) -> Result<R, String> {
        if path.is_directory() {
            return Err("not-a-file".to_string());
        }
        let bytes = path.resolve()?.read(path)?;
        let len = bytes.len() as u64;
        f(&mut Cursor::new(bytes), len)
    }
}
