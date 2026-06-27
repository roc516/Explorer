use explorer_core::EPath;
use explorer_ui::FileEntry;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    DirectoryChanged(EPath),
    PreviewFile(EPath),
    OpenArchive(std::path::PathBuf),
}
