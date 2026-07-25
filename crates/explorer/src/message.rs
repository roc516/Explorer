pub mod input;
pub mod window;

#[derive(Debug, Clone)]
pub enum Message {
    Window(iced::window::Id, window::Message),
    WindowOpened(iced::window::Id, window::Launch),
    WindowClosed(iced::window::Id),
    WindowFocused(iced::window::Id),
    Locale(crate::ui::settings::locale::Message),
    Settings(crate::ui::settings::Message),
}

pub use window::Launch;
