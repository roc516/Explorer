pub mod backends;
mod path;

pub use backends::{
    ensure_backends_registered, is_mounted_path, try_registry, EntryKind, FsBackend, FsRegistry,
};
pub use path::{
    disk_breadcrumbs, disk_path, extension_of, file_name_of, Mounter, PathBreadcrumb, Reader, EPath,
    Volume,
};

pub(crate) use backends::list_drives;
