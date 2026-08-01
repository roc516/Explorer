use std::path::Path;
use std::sync::Arc;

use explorer_core::DirEntry;

use crate::entry::FileEntry;
use crate::i18n::{ids, LanguageBundle};
use super::{ModelError, StatusInfo};

#[derive(Debug, Clone)]
pub struct FileListState {
    pub entries: Vec<FileEntry>,
    pub selected_index: Option<usize>,
    pub loading: bool,
    pub error: Option<ModelError>,
    pub status: StatusInfo,
}

impl FileListState {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            selected_index: None,
            loading: true,
            error: None,
            status: StatusInfo::Loading,
        }
    }

    pub fn begin_load(&mut self) {
        self.loading = true;
        self.error = None;
        self.status = StatusInfo::Loading;
    }

    pub fn select_entry(&mut self, index: usize) {
        self.selected_index = Some(index);
    }

    pub fn select_path(&mut self, path: &Path) {
        self.selected_index = self.entries.iter().position(|entry| entry.path() == path);
    }

    pub fn on_directory_loaded(
        &mut self,
        result: Result<(Arc<dyn DirEntry>, Vec<FileEntry>), String>,
    ) {
        self.loading = false;
        match result {
            Ok((_dir, entries)) => {
                self.entries = entries;
                self.selected_index = None;
                self.error = None;
                self.status = StatusInfo::ItemCount(self.entries.len());
            }
            Err(message) => {
                self.entries.clear();
                self.selected_index = None;
                self.error = Some(ModelError::External(message));
                self.status = StatusInfo::LoadFailed;
            }
        }
    }

    pub fn status_text(&self, bundle: &LanguageBundle) -> String {
        match &self.status {
            StatusInfo::Loading => bundle.tr(ids::STATUS_LOADING),
            StatusInfo::ItemCount(count) => bundle.format_count(*count),
            StatusInfo::LoadFailed => bundle.tr(ids::STATUS_LOAD_FAILED),
            StatusInfo::Opened { name } => bundle.format_opened(name),
            StatusInfo::Path(path) => path.clone(),
        }
    }

    pub fn error_text(&self, bundle: &LanguageBundle) -> Option<String> {
        self.error.as_ref().map(|error| match error {
            ModelError::InvalidPath => bundle.tr(ids::ERROR_INVALID_PATH),
            ModelError::NotDirectory => bundle.tr(ids::ERROR_NOT_DIRECTORY),
            ModelError::External(message) => message.clone(),
        })
    }
}

impl Default for FileListState {
    fn default() -> Self {
        Self::new()
    }
}
