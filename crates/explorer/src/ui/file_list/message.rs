use explorer_core::{BlockDevice, EPath};
use explorer_app::FileEntry;

use super::columns::Column;

#[derive(Debug, Clone)]
pub enum Message {
    EntryClicked(usize),
    EntryDoubleClicked(usize),
    DirectoryLoaded(Result<(EPath, Vec<FileEntry>), String>),
    ColumnResizeStarted(Column),
    ColumnResizeMoved(f32),
    ColumnResizeEnded,
    ColumnSortClicked(Column),
}

#[derive(Debug, Clone)]
pub enum Action {
    /// User navigated into a directory (push history + sync tree).
    Navigated(EPath),
    /// Directory listing finished (sync address bar + tree).
    DirectoryLoaded(EPath),
    PreviewFile(EPath),
    OpenArchive(BlockDevice),
}
