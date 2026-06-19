use std::path::PathBuf;

use explorer_core::FileEntry;

#[derive(Debug, Clone)]
pub enum Message {
    EntryClicked(usize),
    EntryDoubleClicked(usize),
    DirectoryLoaded(Result<(PathBuf, Vec<FileEntry>), String>),
}
