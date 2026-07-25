use std::path::PathBuf;

use explorer_core::filesystem::{BlockDevice, EPath, Mounter};

use crate::entry::FileEntry;
use crate::i18n::{ids, LanguageBundle};
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

#[derive(Debug, Clone)]
pub enum OpenEntryAction {
    Navigate(EPath),
    Preview(EPath),
    OpenArchive(BlockDevice),
    OpenedSystem { name: String },
}

#[derive(Debug, Clone)]
pub struct ExplorerModel {
    pub current_path: EPath,
    pub entries: Vec<FileEntry>,
    pub selected_index: Option<usize>,
    pub loading: bool,
    pub error: Option<ModelError>,
    pub status: StatusInfo,
    pub bundle: LanguageBundle,
}

impl ExplorerModel {
    pub fn new_local() -> Self {
        let initial = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"))
            });
        Self::with_path(EPath::local(initial))
    }

    pub fn new_mounted(device: BlockDevice) -> Self {
        Self::with_path(
            Mounter::mount_root(device).unwrap_or_else(|message| {
                panic!("unsupported archive: {message}")
            }),
        )
    }

    fn with_path(initial_path: EPath) -> Self {
        let bundle = LanguageBundle::new(crate::i18n::Locale::En);

        Self {
            current_path: initial_path,
            entries: Vec::new(),
            selected_index: None,
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

    pub fn can_go_up(&self) -> bool {
        self.current_path.parent().is_some()
    }

    pub fn begin_load(&mut self) {
        self.loading = true;
        self.error = None;
        self.status = StatusInfo::Loading;
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

        self.begin_load();
        Some(path)
    }

    pub fn go_up(&mut self) -> Option<EPath> {
        let parent = self.current_path.parent()?;
        self.navigate(parent)
    }

    pub fn refresh(&mut self) -> Option<EPath> {
        self.begin_load();
        Some(self.current_path.clone())
    }

    pub fn select_entry(&mut self, index: usize) {
        self.selected_index = Some(index);
    }

    pub fn select_path(&mut self, path: &EPath) {
        self.selected_index = self.entries.iter().position(|entry| entry.path == *path);
    }

    pub fn open_entry(&mut self, index: usize) -> Option<OpenEntryAction> {
        let entry = self.entries.get(index)?;

        if entry.is_dir {
            return self
                .navigate(entry.path.clone())
                .map(OpenEntryAction::Navigate);
        }

        if let Some(archive) = entry.path.as_mountable_device() {
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
        match result {
            Ok((path, entries)) => {
                self.current_path = path;
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

    pub fn set_path_error(&mut self, error: ModelError) {
        let message = match &error {
            ModelError::InvalidPath => self.bundle.tr(ids::ERROR_INVALID_PATH),
            ModelError::NotDirectory => self.bundle.tr(ids::ERROR_NOT_DIRECTORY),
            ModelError::External(message) => message.clone(),
        };
        self.error = Some(error);
        self.status = StatusInfo::Path(message);
    }
}
