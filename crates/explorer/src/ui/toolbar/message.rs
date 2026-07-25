use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Message {
    GoUp,
    GoBack,
    GoForward,
    Refresh,
    AddressEdited(String),
    AddressSubmit,
    AddressEditStart,
    BreadcrumbNavigate(PathBuf),
}

#[derive(Debug, Clone)]
pub enum Action {
    Load(PathBuf),
}
