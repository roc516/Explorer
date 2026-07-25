use explorer_core::EPath;

#[derive(Debug, Clone)]
pub enum Message {
    GoUp,
    GoBack,
    GoForward,
    Refresh,
    AddressEdited(String),
    AddressSubmit,
    AddressEditStart,
    BreadcrumbNavigate(EPath),
}

#[derive(Debug, Clone)]
pub enum Action {
    Load(EPath),
}
