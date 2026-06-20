use std::path::PathBuf;

pub trait BackendBootstrap {
    fn list_roots(&self) -> Vec<PathBuf> {
        Vec::new()
    }

    fn default_initial_path(&self) -> Option<PathBuf> {
        None
    }
}
