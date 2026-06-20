pub mod backends;
mod path;

pub use backends::{
    ensure_backends_registered, is_mounted_path, try_registry, ArchiveMount, BackendBootstrap,
    BackendIdentity, EntryKind, FsBackend, FsIo, FsRegistry, PathMetadata, PathNavigation,
};
pub use path::{
    disk_breadcrumbs, disk_path, extension_of, file_name_of, path_breadcrumbs, Mounter,
    PathBreadcrumb, Reader, EPath,
};

pub(crate) use backends::{default_initial_path, list_drives};
pub(crate) use path::{from_address_input, parent_path};

use crate::entry::FileEntry;

pub fn read_directory(path: &EPath) -> Result<Vec<FileEntry>, String> {
    Reader::read_directory(path)
}