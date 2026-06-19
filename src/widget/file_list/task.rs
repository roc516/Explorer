use std::path::PathBuf;

use iced::Task;

use super::message::Message;

pub fn load_directory_task(path: PathBuf) -> Task<Message> {
    use explorer_core::read_directory;
    Task::perform(
        async move { read_directory(&path).map(|entries| (path, entries)) },
        Message::DirectoryLoaded,
    )
}
