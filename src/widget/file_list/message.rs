use explorer_core::{BrowsePath, FileEntry};

use super::columns::Column;

#[derive(Debug, Clone)]
pub enum Message {
    EntryClicked(usize),
    EntryDoubleClicked(usize),
    DirectoryLoaded(Result<(BrowsePath, Vec<FileEntry>), String>),
    ColumnResizeStarted(Column),
    ColumnResizeMoved(f32),
    ColumnResizeEnded,
    ColumnSortClicked(Column),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    DirectoryChanged(BrowsePath),
    PreviewFile(BrowsePath),
    OpenArchive(std::path::PathBuf),
}
