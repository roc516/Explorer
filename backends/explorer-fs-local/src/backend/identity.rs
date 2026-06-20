use explorer_core::filesystem::BackendIdentity;

use super::LocalBackend;

pub const ID: &str = "local";

impl BackendIdentity for LocalBackend {
    fn id(&self) -> &'static str {
        ID
    }

    fn is_disk_backend(&self) -> bool {
        true
    }
}
