use std::path::{Path, PathBuf};

use super::EntryKind;
use crate::filesystem::path::PathOps;

pub trait PathMetadata {
    fn exists(&self, path: &PathOps) -> bool;
    fn is_file(&self, path: &PathOps) -> bool;
    fn is_directory(&self, path: &PathOps) -> bool;
    fn file_name(&self, path: &PathOps) -> String;
    fn extension(&self, path: &PathOps) -> Option<String>;
    fn preview_path(&self, path: &PathOps) -> PathBuf;

    fn entry_kind(&self, _container: &Path, _inner: &Path) -> Option<EntryKind> {
        None
    }
}
