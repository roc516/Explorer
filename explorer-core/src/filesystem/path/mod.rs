mod access;
mod address;
mod builders;
mod mounter;
mod ops;
mod reader;
mod util;

pub use builders::{disk_breadcrumbs, PathBreadcrumb};
pub use mounter::Mounter;
pub use ops::{disk_path, EPath};
pub use reader::Reader;
pub use util::{extension_of, file_name_of};

pub(crate) use address::{from_address_input, parent_path};

pub fn path_breadcrumbs(path: &EPath) -> Vec<PathBreadcrumb> {
    path.breadcrumbs()
}
