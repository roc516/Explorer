mod archive;
mod bootstrap;
mod identity;
mod io;
mod metadata;

pub use identity::{EXTENSIONS, ID};

pub struct ZipBackend;

impl explorer_core::filesystem::FsBackend for ZipBackend {}
