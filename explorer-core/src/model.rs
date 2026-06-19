use std::path::PathBuf;

use crate::entry::FileEntry;
use crate::fs::{default_initial_path, open_with_system, parent_path};
use crate::i18n::{ids, LanguageBundle};
use crate::navigation::NavigationHistory;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelError {
    InvalidPath,
    NotDirectory,
    External(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatusInfo {
    Loading,
    ItemCount(usize),
    LoadFailed,
    Opened { name: String },
    Path(String),
}

#[derive(Debug, Clone)]
pub struct ExplorerModel {
    pub current_path: PathBuf,
    pub entries: Vec<FileEntry>,
    pub selected_index: Option<usize>,
    pub address_input: String,
    pub navigation: NavigationHistory,
    pub loading: bool,
    pub error: Option<ModelError>,
    pub status: StatusInfo,
    pub bundle: LanguageBundle,
}

impl ExplorerModel {
    pub fn new() -> Self {
        let initial_path = default_initial_path();
        let bundle = LanguageBundle::new(crate::i18n::Locale::En);

        Self {
            current_path: initial_path.clone(),
            entries: Vec::new(),
            selected_index: None,
            address_input: initial_path.display().to_string(),
            navigation: NavigationHistory::new(initial_path),
            loading: true,
            error: None,
            status: StatusInfo::Loading,
            bundle,
        }
    }

    pub fn set_locale(&mut self, locale: crate::i18n::Locale) {
        self.bundle = LanguageBundle::new(locale);
    }

    pub fn status_text(&self) -> String {
        match &self.status {
            StatusInfo::Loading => self.bundle.tr(ids::STATUS_LOADING),
            StatusInfo::ItemCount(count) => self.bundle.format_count(*count),
            StatusInfo::LoadFailed => self.bundle.tr(ids::STATUS_LOAD_FAILED),
            StatusInfo::Opened { name } => self.bundle.format_opened(name),
            StatusInfo::Path(path) => path.clone(),
        }
    }

    pub fn error_text(&self) -> Option<String> {
        self.error.as_ref().map(|error| match error {
            ModelError::InvalidPath => self.bundle.tr(ids::ERROR_INVALID_PATH),
            ModelError::NotDirectory => self.bundle.tr(ids::ERROR_NOT_DIRECTORY),
            ModelError::External(message) => message.clone(),
        })
    }

    pub fn can_go_back(&self) -> bool {
        self.navigation.can_go_back()
    }

    pub fn can_go_forward(&self) -> bool {
        self.navigation.can_go_forward()
    }

    pub fn can_go_up(&self) -> bool {
        parent_path(&self.current_path).is_some()
    }

    pub fn navigate(&mut self, path: PathBuf) -> Option<PathBuf> {
        if !path.exists() {
            self.error = Some(ModelError::InvalidPath);
            self.status = StatusInfo::Path(self.bundle.tr(ids::ERROR_INVALID_PATH));
            return None;
        }

        if !path.is_dir() {
            self.error = Some(ModelError::NotDirectory);
            self.status = StatusInfo::Path(self.bundle.tr(ids::ERROR_NOT_DIRECTORY));
            return None;
        }

        self.loading = true;
        self.error = None;
        self.status = StatusInfo::Loading;
        self.navigation.push(path.clone());
        Some(path)
    }

    pub fn go_up(&mut self) -> Option<PathBuf> {
        let parent = parent_path(&self.current_path)?;
        self.navigate(parent)
    }

    pub fn go_back(&mut self) -> Option<PathBuf> {
        let path = self.navigation.go_back()?;
        self.loading = true;
        self.error = None;
        self.status = StatusInfo::Loading;
        Some(path)
    }

    pub fn go_forward(&mut self) -> Option<PathBuf> {
        let path = self.navigation.go_forward()?;
        self.loading = true;
        self.error = None;
        self.status = StatusInfo::Loading;
        Some(path)
    }

    pub fn refresh(&mut self) -> Option<PathBuf> {
        self.loading = true;
        self.error = None;
        self.status = StatusInfo::Loading;
        Some(self.current_path.clone())
    }

    pub fn set_address(&mut self, value: String) {
        self.address_input = value;
    }

    pub fn submit_address(&mut self) -> Option<PathBuf> {
        let path = PathBuf::from(self.address_input.trim());
        self.navigate(path)
    }

    pub fn select_entry(&mut self, index: usize) {
        self.selected_index = Some(index);
    }

    pub fn open_entry(&mut self, index: usize) -> Option<PathBuf> {
        let entry = self.entries.get(index)?;

        if entry.is_dir {
            return self.navigate(entry.path.clone());
        }

        match open_with_system(&entry.path) {
            Ok(()) => {
                self.status = StatusInfo::Opened {
                    name: entry.name.clone(),
                };
                None
            }
            Err(message) => {
                self.error = Some(ModelError::External(message));
                self.status = StatusInfo::LoadFailed;
                None
            }
        }
    }

    pub fn on_directory_loaded(
        &mut self,
        result: Result<(PathBuf, Vec<FileEntry>), String>,
    ) {
        self.loading = false;
        match result {
            Ok((path, entries)) => {
                self.current_path = path.clone();
                self.address_input = path.display().to_string();
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
}
