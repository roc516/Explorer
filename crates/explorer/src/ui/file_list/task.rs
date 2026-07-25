use explorer_core::{DirEntry, EPath, Reader};
use explorer_app::FileEntry;
use iced::Task;

use super::message::Message;

/// Load a directory listing via a retained [`DirEntry`] handle.
pub fn load_directory_from_dir(dir: DirEntry) -> Task<Message> {
    let path = dir.path.clone();
    Task::perform(
        async move {
            dir.list().map(|entries| {
                let entries: Vec<FileEntry> = entries.into_iter().map(FileEntry::from_fs).collect();
                (path, entries)
            })
        },
        Message::DirectoryLoaded,
    )
}

/// Load a directory listing via a full [`EPath`] (toolbar / history / tree).
pub fn load_directory_task(path: EPath) -> Task<Message> {
    Task::perform(
        async move {
            let nav = path.navigation_path();
            Reader::read_directory(&path).map(|entries| {
                let entries: Vec<FileEntry> = entries.into_iter().map(FileEntry::from_fs).collect();
                (nav, entries)
            })
        },
        Message::DirectoryLoaded,
    )
}
