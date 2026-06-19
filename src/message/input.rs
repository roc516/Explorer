use iced::keyboard;

#[derive(Debug, Clone)]
pub enum Message {
    KeyPressed(keyboard::Key, keyboard::Modifiers),
}
