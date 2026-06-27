use crate::filesystem::Volume;

pub trait BackendBootstrap {
    fn list_roots(&self) -> Vec<Volume> {
        Vec::new()
    }
}
