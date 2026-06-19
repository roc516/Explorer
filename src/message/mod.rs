pub mod input;
pub mod preview;
pub mod settings;
pub mod theme;
pub mod window;

#[derive(Debug, Clone)]
pub enum Message {
    Window(iced::window::Id, window::Message),
    WindowOpened(iced::window::Id, window::Launch),
    WindowClosed(iced::window::Id),
    WindowFocused(iced::window::Id),
    Theme(theme::Message),
    Locale(crate::widget::settings_dialog::locale::Message),
    Settings(settings::Message),
}

pub use window::Launch;
