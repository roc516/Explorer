pub mod input;
pub mod preview;
pub mod settings;
pub mod theme;

use crate::widget::{directory_tree, file_list, settings_dialog, toolbar};

#[derive(Debug, Clone)]
pub enum Message {
    Explorer(toolbar::Message),
    FileList(file_list::Message),
    Tree(directory_tree::Message),
    Theme(theme::Message),
    Locale(settings_dialog::locale::Message),
    Settings(settings::Message),
    Preview(preview::Message),
    Input(input::Message),
}
