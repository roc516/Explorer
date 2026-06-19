use std::path::PathBuf;
use std::time::SystemTime;

use crate::browse_path::BrowsePath;
use crate::i18n::{ids, LanguageBundle};

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: BrowsePath,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<SystemTime>,
}

impl FileEntry {
    pub fn type_label(&self, bundle: &LanguageBundle) -> String {
        if self.is_dir {
            return bundle.tr(ids::ENTRY_FOLDER);
        }

        let extension = self
            .path
            .local_file()
            .and_then(|path| path.extension())
            .or_else(|| std::path::Path::new(&self.name).extension())
            .and_then(|ext| ext.to_str());
        extension
            .map(|ext| bundle.format_file_type(ext))
            .unwrap_or_else(|| bundle.tr(ids::ENTRY_FILE))
    }

    pub fn size_label(&self, bundle: &LanguageBundle) -> String {
        if self.is_dir {
            return String::new();
        }
        bundle.format_size(self.size)
    }

    pub fn modified_label(&self, bundle: &LanguageBundle) -> String {
        self.modified
            .map(|time| bundle.format_datetime(time))
            .unwrap_or_default()
    }
}
