use explorer_core::BrowsePath;
use iced::Task;

use super::message::Message;

pub fn load_directory_task(path: BrowsePath) -> Task<Message> {
    Task::perform(
        async move { explorer_core::read_directory(&path).map(|entries| (path, entries)) },
        Message::DirectoryLoaded,
    )
}
