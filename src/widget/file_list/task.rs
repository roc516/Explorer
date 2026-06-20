use explorer_core::EPath;
use iced::Task;

use super::message::Message;

pub fn load_directory_task(path: EPath) -> Task<Message> {
    Task::perform(
        async move { explorer_core::read_directory(&path).map(|entries| (path, entries)) },
        Message::DirectoryLoaded,
    )
}