mod archive;
mod bootstrap;
mod identity;
mod io;
mod metadata;
mod navigation;

pub use identity::ID;

pub struct LocalBackend;

impl explorer_core::filesystem::FsBackend for LocalBackend {}
