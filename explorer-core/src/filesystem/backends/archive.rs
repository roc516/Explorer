use std::path::Path;
use std::sync::Arc;

use super::MountSession;

pub trait ArchiveMount {
    fn open_session(&self, container: &Path) -> Result<Arc<dyn MountSession>, String> {
        let _ = container;
        Err("archive-session-not-supported".to_string())
    }

    fn close_session(&self, _container: &Path) {}
}
