use std::path::Path;

pub trait BackendIdentity {
    fn id(&self) -> &'static str;

    fn is_disk_backend(&self) -> bool {
        false
    }

    fn matches(&self, _path: &Path) -> bool {
        false
    }
}
