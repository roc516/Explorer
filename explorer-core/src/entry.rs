use std::time::SystemTime;

use crate::filesystem::EPath;
use crate::i18n::{ids, LanguageBundle};

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: EPath,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<SystemTime>,
}

impl FileEntry {
    pub fn type_label(&self, bundle: &LanguageBundle) -> String {
        if self.is_dir {
            return bundle.tr(ids::ENTRY_FOLDER);
        }

        let extension = self.path.extension().or_else(|| {
            std::path::Path::new(&self.name)
                .extension()
                .and_then(|ext| ext.to_str())
                .map(str::to_ascii_lowercase)
        });
        extension
            .as_deref()
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
