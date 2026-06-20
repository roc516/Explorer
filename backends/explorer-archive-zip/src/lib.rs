mod archive;
mod backend;
mod path;

pub use backend::{EXTENSIONS, ID};

use explorer_core::filesystem::FsRegistry;

pub fn register(registry: &mut FsRegistry) {
    registry.register(Box::new(backend::ZipBackend));
}
