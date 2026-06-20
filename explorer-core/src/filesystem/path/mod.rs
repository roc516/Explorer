mod address;
mod builders;
mod ops;
mod util;

pub use builders::{disk_breadcrumbs, join_mounted_inner, PathBreadcrumb};
pub use ops::{disk_path, mount_path, PathOps};
pub use util::{extension_of, file_name_of};

pub(crate) use address::{from_address_input, parent_path};

pub fn path_breadcrumbs(path: &PathOps) -> Vec<PathBreadcrumb> {
    path.breadcrumbs()
}
