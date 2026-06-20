mod backend;
mod directory;

pub use backend::ID;

use explorer_core::filesystem::FsRegistry;

pub fn register(registry: &mut FsRegistry) {
    registry.register(Box::new(backend::LocalBackend));
}
