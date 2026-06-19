use std::path::PathBuf;

use crate::message::{preview, input};
use crate::widget::{directory_tree, file_list, toolbar};

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
    Archive(PathBuf),
}
