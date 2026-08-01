use std::path::Path;
use std::sync::Arc;
use std::time::SystemTime;

use explorer_core::{DirEntry, FileEntry as CoreFileEntry, FsEntry};

use crate::i18n::{ids, LanguageBundle};

/// UI-level listing entry wrapping a core [`FsEntry`].
#[derive(Debug, Clone)]
pub struct FileEntry {
    inner: FsEntry,
}

impl FileEntry {
    pub fn from_fs(entry: FsEntry) -> Self {
        Self { inner: entry }
    }

    pub fn path(&self) -> &Path {
        match &self.inner {
            FsEntry::Dir(d) => d.path(),
            FsEntry::File(f) => f.path(),
            FsEntry::Volume(_) => Path::new(""),
        }
    }

    pub fn name(&self) -> &str {
        match &self.inner {
            FsEntry::Dir(d) => d.name(),
            FsEntry::File(f) => f.name(),
            FsEntry::Volume(v) => v.name(),
        }
    }

    pub fn is_dir(&self) -> bool {
        matches!(self.inner, FsEntry::Dir(_))
    }

    pub fn as_dir(&self) -> Option<&Arc<dyn DirEntry>> {
        match &self.inner {
            FsEntry::Dir(d) => Some(d),
            FsEntry::File(_) => None,
            FsEntry::Volume(_) => None,
        }
    }

    pub fn size(&self) -> u64 {
        match &self.inner {
            FsEntry::File(f) => f.size(),
            FsEntry::Dir(_) => 0,
            FsEntry::Volume(_) => 0,
        }
    }

    pub fn modified(&self) -> Option<SystemTime> {
        match &self.inner {
            FsEntry::File(f) => f.modified(),
            FsEntry::Dir(_) => None,
            FsEntry::Volume(_) => None,
        }
    }

    pub fn as_file(&self) -> Option<Arc<dyn CoreFileEntry>> {
        match &self.inner {
            FsEntry::File(f) => Some(f.clone()),
            FsEntry::Dir(_) => None,
            FsEntry::Volume(_) => None,
        }
    }

    pub fn fs_entry(&self) -> &FsEntry {
        &self.inner
    }

    pub fn type_label(&self, bundle: &LanguageBundle) -> String {
        if self.is_dir() {
            return bundle.tr(ids::ENTRY_FOLDER);
        }

        let extension = self.path()
            .extension()
            .and_then(|ext| ext.to_str())
            .map(str::to_ascii_lowercase)
            .or_else(|| {
                Path::new(self.name())
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
        if self.is_dir() {
            return String::new();
        }
        bundle.format_size(self.size())
    }

    pub fn modified_label(&self, bundle: &LanguageBundle) -> String {
        self.modified()
            .map(|time| bundle.format_datetime(time))
            .unwrap_or_default()
    }
}
