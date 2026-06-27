use std::path::Path;

use super::EntryKind;
use crate::filesystem::path::EPath;

pub trait PathMetadata {
    fn exists(&self, path: &EPath) -> bool;

    fn entry_kind(&self, _container: &Path, _inner: &Path) -> Option<EntryKind> {
        None
    }
}
