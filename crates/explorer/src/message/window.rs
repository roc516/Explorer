use explorer_core::BlockDevice;

use crate::message::input;
use crate::ui::{directory_tree, file_list, preview, toolbar};

#[derive(Debug, Clone)]
pub enum Message {
    Explorer(toolbar::Message),
    FileList(file_list::Message),
    Tree(directory_tree::Message),
    Preview(preview::Message),
    Input(input::Message),
}

#[derive(Debug, Clone)]
pub enum Launch {
    Local,
    Archive(BlockDevice),
}
