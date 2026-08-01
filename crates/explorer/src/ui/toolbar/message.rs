use std::sync::Arc;

use explorer_core::DirEntry;

#[derive(Debug, Clone)]
pub enum Message {
    GoUp,
    GoBack,
    GoForward,
    Refresh,
    AddressEdited(String),
    AddressSubmit,
    AddressEditStart,
    BreadcrumbNavigate(std::path::PathBuf),
}

#[derive(Debug, Clone)]
pub enum Action {
    Load(Arc<dyn DirEntry>),
}
