use iced::theme::Mode;

use crate::theme::AppTheme;

#[derive(Debug, Clone)]
pub enum Message {
    Selected(AppTheme),
    SystemChanged(Mode),
}
