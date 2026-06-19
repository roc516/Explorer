use explorer_core::Language;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Message {
    Selected(Language),
}
