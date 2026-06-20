use std::path::PathBuf;

use crate::entry::FileEntry;
use crate::filesystem::{default_initial_path, from_address_input, parent_path, Mounter, EPath};
use crate::i18n::{ids, LanguageBundle};
use crate::navigation::NavigationHistory;
use crate::preview;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenEntryAction {
    Navigate(EPath),
    Preview(EPath),
    OpenArchive(PathBuf),
    OpenedSystem { name: String },
}

#[derive(Debug, Clone)]
pub struct ExplorerModel {
    pub current_path: EPath,
    pub entries: Vec<FileEntry>,
    pub selected_index: Option<usize>,
    pub address_input: String,
    pub address_editing: bool,
    pub reveal_path: Option<EPath>,
    pub navigation: NavigationHistory,
    pub loading: bool,
    pub error: Option<ModelError>,
    pub status: StatusInfo,
    pub bundle: LanguageBundle,
}

impl ExplorerModel {
    pub fn new_local() -> Self {
        Self::with_path(EPath::local(default_initial_path()))
    }

    pub fn new_mounted(container: PathBuf) -> Self {
        Self::with_path(
            Mounter::mount_root(container).unwrap_or_else(|message| {
                panic!("unsupported archive: {message}")
            }),
        )
    }

    fn with_path(initial_path: EPath) -> Self {
        let bundle = LanguageBundle::new(crate::i18n::Locale::En);

        Self {
            current_path: initial_path.clone(),
            entries: Vec::new(),
            selected_index: None,
            address_input: initial_path.display(),
            address_editing: false,
            reveal_path: None,
            navigation: NavigationHistory::new(initial_path),
            loading: true,
            error: None,
            status: StatusInfo::Loading,
            bundle,
        }
    }

    pub fn new() -> Self {
        Self::new_local()
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

    pub fn navigate(&mut self, path: EPath) -> Option<EPath> {
        if !path.exists() {
            self.error = Some(ModelError::InvalidPath);
            self.status = StatusInfo::Path(self.bundle.tr(ids::ERROR_INVALID_PATH));
            return None;
        }

        if !path.is_directory() {
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

    pub fn go_up(&mut self) -> Option<EPath> {
        let parent = parent_path(&self.current_path)?;
        self.navigate(parent)
    }

    pub fn go_back(&mut self) -> Option<EPath> {
        let path = self.navigation.go_back()?;
        self.loading = true;
        self.error = None;
        self.status = StatusInfo::Loading;
        Some(path)
    }

    pub fn go_forward(&mut self) -> Option<EPath> {
        let path = self.navigation.go_forward()?;
        self.loading = true;
        self.error = None;
        self.status = StatusInfo::Loading;
        Some(path)
    }

    pub fn refresh(&mut self) -> Option<EPath> {
        self.loading = true;
        self.error = None;
        self.status = StatusInfo::Loading;
        Some(self.current_path.clone())
    }

    pub fn set_address(&mut self, value: String) {
        self.address_input = value;
    }

    pub fn start_address_edit(&mut self) {
        self.address_editing = true;
        self.address_input = self.current_path.display();
    }

    pub fn cancel_address_edit(&mut self) {
        self.address_editing = false;
        self.address_input = self.current_path.display();
    }

    pub fn submit_address(&mut self) -> Option<EPath> {
        self.address_editing = false;
        let path = from_address_input(&self.address_input, &self.current_path);

        if !path.exists() {
            self.error = Some(ModelError::InvalidPath);
            self.status = StatusInfo::Path(self.bundle.tr(ids::ERROR_INVALID_PATH));
            return None;
        }

        if path.is_directory() {
            self.reveal_path = None;
            return self.navigate(path);
        }

        if path.is_file() {
            let Some(parent) = path.parent() else {
                self.error = Some(ModelError::InvalidPath);
                self.status = StatusInfo::Path(self.bundle.tr(ids::ERROR_INVALID_PATH));
                return None;
            };

            if parent == self.current_path {
                self.address_input = path.display();
                self.error = None;
                self.selected_index = self
                    .entries
                    .iter()
                    .position(|entry| entry.path == path);
                self.status = StatusInfo::ItemCount(self.entries.len());
                return None;
            }

            self.reveal_path = Some(path);
            return self.navigate(parent);
        }

        self.error = Some(ModelError::InvalidPath);
        self.status = StatusInfo::Path(self.bundle.tr(ids::ERROR_INVALID_PATH));
        None
    }

    pub fn select_entry(&mut self, index: usize) {
        self.selected_index = Some(index);
    }

    pub fn open_entry(&mut self, index: usize) -> Option<OpenEntryAction> {
        let entry = self.entries.get(index)?;

        if entry.is_dir {
            return self
                .navigate(entry.path.clone())
                .map(OpenEntryAction::Navigate);
        }

        if let Some(archive) = entry.path.nested_archive_file() {
            return Some(OpenEntryAction::OpenArchive(archive));
        }

        if preview::is_previewable(&entry.path) {
            return Some(OpenEntryAction::Preview(entry.path.clone()));
        }

        match entry.path.open_with_system() {
            Ok(()) => {
                self.status = StatusInfo::Opened {
                    name: entry.name.clone(),
                };
                Some(OpenEntryAction::OpenedSystem {
                    name: entry.name.clone(),
                })
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
        result: Result<(EPath, Vec<FileEntry>), String>,
    ) {
        self.loading = false;
        self.address_editing = false;
        match result {
            Ok((path, entries)) => {
                self.current_path = path.clone();
                if let Some(reveal) = self.reveal_path.take() {
                    self.address_input = reveal.display();
                    self.selected_index = entries
                        .iter()
                        .position(|entry| entry.path == reveal);
                } else {
                    self.address_input = path.display();
                    self.selected_index = None;
                }
                self.entries = entries;
                self.error = None;
                self.status = StatusInfo::ItemCount(self.entries.len());
            }
            Err(message) => {
                self.reveal_path = None;
                self.entries.clear();
                self.selected_index = None;
                self.error = Some(ModelError::External(message));
                self.status = StatusInfo::LoadFailed;
            }
        }
    }
}
