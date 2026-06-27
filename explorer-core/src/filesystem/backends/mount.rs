use std::any::Any;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use super::FsBackend;

pub trait MountSession: Send + Sync + Any {}

type SessionKey = (&'static str, PathBuf);

static SESSIONS: OnceLock<Mutex<HashMap<SessionKey, Arc<dyn MountSession>>>> = OnceLock::new();

fn sessions() -> &'static Mutex<HashMap<SessionKey, Arc<dyn MountSession>>> {
    SESSIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(crate) fn ensure_session(
    backend: &dyn FsBackend,
    container: &Path,
) -> Result<Arc<dyn MountSession>, String> {
    let key = (backend.id(), container.to_path_buf());
    let mut guard = sessions().lock().expect("mount sessions poisoned");
    if let Some(session) = guard.get(&key) {
        return Ok(session.clone());
    }

    let session = backend.open(container)?;
    guard.insert(key, session.clone());
    Ok(session)
}

pub(crate) fn get_session(backend_id: &'static str, container: &Path) -> Option<Arc<dyn MountSession>> {
    let key = (backend_id, container.to_path_buf());
    sessions()
        .lock()
        .expect("mount sessions poisoned")
        .get(&key)
        .cloned()
}

pub(crate) fn remove_session(backend_id: &'static str, container: &Path) {
    let key = (backend_id, container.to_path_buf());
    sessions()
        .lock()
        .expect("mount sessions poisoned")
        .remove(&key);
}
