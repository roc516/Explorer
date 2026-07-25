use explorer_core::DirEntry;
use explorer_app::FileEntry;
use iced::Task;

use super::message::Message;

/// Load a directory listing via a retained [`DirEntry`] handle.
pub fn load_directory_from_dir(dir: DirEntry) -> Task<Message> {
    Task::perform(
        async move {
            dir.list().map(|entries| {
                let entries: Vec<FileEntry> = entries.into_iter().map(FileEntry::from_fs).collect();
                (dir, entries)
            })
        },
        Message::DirectoryLoaded,
    )
}
