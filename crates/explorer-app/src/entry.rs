use std::time::SystemTime;

use explorer_core::filesystem::{Mounter, EPath};
use explorer_core::{FileEntry as CoreFileEntry, FsEntry};

use crate::i18n::{ids, LanguageBundle};

/// UI-level listing entry: full [`EPath`] plus the original core [`FsEntry`].
#[derive(Debug, Clone)]
pub struct FileEntry {
    path: EPath,
    inner: FsEntry,
}

impl FileEntry {
    /// Wrap a core listing entry under `dir`, attaching a full [`EPath`].
    pub fn from_fs(entry: FsEntry, dir: &EPath) -> Self {
        let relative = match &entry {
            FsEntry::Dir(d) => d.path.clone(),
            FsEntry::File(f) => f.path.clone(),
        };
        let path = if Mounter::is_mount(dir) {
            Mounter::mount_path(dir.root().clone(), relative, dir.backend())
        } else {
            EPath::local(relative)
        };
        Self { path, inner: entry }
    }

    pub fn path(&self) -> &EPath {
        &self.path
    }

    pub fn name(&self) -> &str {
        match &self.inner {
            FsEntry::Dir(d) => d.name.as_str(),
            FsEntry::File(f) => f.name.as_str(),
        }
    }

    pub fn is_dir(&self) -> bool {
        matches!(self.inner, FsEntry::Dir(_))
    }

    pub fn size(&self) -> u64 {
        match &self.inner {
            FsEntry::File(f) => f.size,
            FsEntry::Dir(_) => 0,
        }
    }

    pub fn modified(&self) -> Option<SystemTime> {
        match &self.inner {
            FsEntry::File(f) => f.modified,
            FsEntry::Dir(_) => None,
        }
    }

    pub fn as_file(&self) -> Option<&CoreFileEntry> {
        match &self.inner {
            FsEntry::File(f) => Some(f),
            FsEntry::Dir(_) => None,
        }
    }

    pub fn fs_entry(&self) -> &FsEntry {
        &self.inner
    }

    pub fn type_label(&self, bundle: &LanguageBundle) -> String {
        if self.is_dir() {
            return bundle.tr(ids::ENTRY_FOLDER);
        }

        let extension = self.path.extension().or_else(|| {
            std::path::Path::new(self.name())
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
