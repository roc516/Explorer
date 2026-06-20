pub mod backends;
mod path;

pub use backends::{
    ensure_backends_registered, is_mounted_path, try_registry, ArchiveMount, BackendBootstrap,
    BackendIdentity, EntryKind, FsBackend, FsIo, FsRegistry, PathMetadata, PathNavigation,
};
pub use path::{
    disk_breadcrumbs, disk_path, extension_of, file_name_of, Mounter, PathBreadcrumb, Reader, EPath,
};

pub(crate) use backends::{default_initial_path, list_drives};
