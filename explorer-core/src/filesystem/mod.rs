pub mod backends;
mod path;

pub use backends::{
    ensure_backends_registered, is_mounted_path, try_registry, EntryKind, FsBackend, FsRegistry,
    MountedDevice,
};
pub use path::{
    disk_path, extension_of, file_name_of, Mounter, Reader, EPath, Volume,
};

pub use backends::list_drives;
