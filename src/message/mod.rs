pub mod explorer;
pub mod file_list;
pub mod input;
pub mod locale;
pub mod settings;
pub mod theme;
pub mod tree;

#[derive(Debug, Clone)]
pub enum Message {
    Explorer(explorer::Message),
    FileList(file_list::Message),
    Tree(tree::Message),
    Theme(theme::Message),
    Locale(locale::Message),
    Settings(settings::Message),
    Input(input::Message),
}
