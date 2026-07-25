use explorer_core::{BlockDevice, DirEntry, FsEntry};
use explorer_app::FileEntry;

use super::columns::Column;

#[derive(Debug, Clone)]
pub enum Message {
    EntryClicked(usize),
    EntryDoubleClicked(usize),
    DirectoryLoaded(Result<(DirEntry, Vec<FileEntry>), String>),
    EntriesSorted {
        id: u64,
        entries: Vec<FileEntry>,
        selected_index: Option<usize>,
    },
    ColumnResizeStarted(Column),
    ColumnResizeMoved(f32),
    ColumnResizeEnded,
    ColumnSortClicked(Column),
}

#[derive(Debug, Clone)]
pub enum Action {
    /// User navigated into a directory (push history + sync tree).
    Navigated(std::path::PathBuf),
    /// Directory listing finished (sync address bar + tree).
    DirectoryLoaded(std::path::PathBuf),
    PreviewFile(FsEntry),
    OpenArchive(BlockDevice),
}
