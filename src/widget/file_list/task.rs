use explorer_core::{EPath, Reader};
use explorer_ui::FileEntry;
use iced::Task;

use super::message::Message;

pub fn load_directory_task(path: EPath) -> Task<Message> {
    Task::perform(
        async move {
            Reader::read_directory(&path)
                .map(|entries| {
                    let entries: Vec<FileEntry> =
                        entries.into_iter().map(FileEntry::from).collect();
                    (path, entries)
                })
        },
        Message::DirectoryLoaded,
    )
}
