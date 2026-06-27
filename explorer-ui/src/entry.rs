use std::time::SystemTime;

use explorer_core::filesystem::EPath;
use explorer_core::FsEntry;

use crate::i18n::{ids, LanguageBundle};

/// UI-level file entry used by the model and widgets.
/// Converted from the core `FsEntry` enum.
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: EPath,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<SystemTime>,
}

impl From<FsEntry> for FileEntry {
    fn from(entry: FsEntry) -> Self {
        match entry {
            FsEntry::Dir(d) => Self {
                name: d.name,
                path: d.path,
                is_dir: true,
                size: 0,
                modified: None,
            },
            FsEntry::File(f) => Self {
                name: f.name,
                path: f.path,
                is_dir: false,
                size: f.size,
                modified: f.modified,
            },
        }
    }
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
