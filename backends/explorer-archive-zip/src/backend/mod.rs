mod archive;
mod bootstrap;
mod identity;
mod io;
mod metadata;
mod navigation;

pub use identity::{EXTENSIONS, ID};

pub struct ZipBackend;

impl explorer_core::filesystem::FsBackend for ZipBackend {}
