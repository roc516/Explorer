use explorer_core::{EPath, Reader};
use iced::Task;

use super::message::Message;

pub fn load_directory_task(path: EPath) -> Task<Message> {
    Task::perform(
        async move { Reader::read_directory(&path).map(|entries| (path, entries)) },
        Message::DirectoryLoaded,
    )
}