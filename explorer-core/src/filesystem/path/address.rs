use super::mounter::Mounter;
use super::ops::EPath;

pub(crate) fn from_address_input(input: &str, context: &EPath) -> EPath {
    Mounter::from_mount_address(input, context).unwrap_or_else(|| EPath::local(input.trim()))
}

pub(crate) fn parent_path(path: &EPath) -> Option<EPath> {
    path.parent()
}
