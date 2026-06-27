use std::path::PathBuf;

use crate::filesystem::Volume;

pub trait BackendBootstrap {
    fn list_roots(&self) -> Vec<Volume> {
        Vec::new()
    }

    fn default_initial_path(&self) -> Option<PathBuf> {
        None
    }
}
