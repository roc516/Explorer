use std::path::PathBuf;

use explorer_core::FileEntry;

use super::columns::Column;

#[derive(Debug, Clone)]
pub enum Message {
    EntryClicked(usize),
    EntryDoubleClicked(usize),
    DirectoryLoaded(Result<(PathBuf, Vec<FileEntry>), String>),
    ColumnResizeStarted(Column),
    ColumnResizeMoved(f32),
    ColumnResizeEnded,
    ColumnSortClicked(Column),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    DirectoryChanged(std::path::PathBuf),
}
