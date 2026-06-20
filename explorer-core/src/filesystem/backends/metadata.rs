use std::path::{Path, PathBuf};

use super::EntryKind;
use crate::filesystem::path::EPath;

pub trait PathMetadata {
    fn exists(&self, path: &EPath) -> bool;
    fn is_file(&self, path: &EPath) -> bool;
    fn is_directory(&self, path: &EPath) -> bool;
    fn file_name(&self, path: &EPath) -> String;
    fn extension(&self, path: &EPath) -> Option<String>;
    fn preview_path(&self, path: &EPath) -> PathBuf;

    fn entry_kind(&self, _container: &Path, _inner: &Path) -> Option<EntryKind> {
        None
    }
}
