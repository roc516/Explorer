use std::path::{Path, PathBuf};

use explorer_core::filesystem::{BlockDevice, Mounter, EPath};
use explorer_core::DirEntry;

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
    /// Navigate into a listed directory (load via [`DirEntry::list`]).
    Navigate(DirEntry),
    Preview(explorer_core::FsEntry),
    OpenArchive(BlockDevice),
    OpenedSystem { name: String },
}

#[derive(Debug, Clone)]
pub struct ExplorerModel {
    /// In-window navigation path (disk absolute, or relative to mount root).
    pub current_path: PathBuf,
    /// Full location for IO / mount identity (`path` mirrors [`Self::current_path`]).
    location: EPath,
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
        Self::with_location(EPath::local(initial))
    }

    pub fn new_mounted(device: BlockDevice) -> Self {
        Self::with_location(
            Mounter::mount_root(device).unwrap_or_else(|message| {
                panic!("unsupported archive: {message}")
            }),
        )
    }

    fn with_location(location: EPath) -> Self {
        let bundle = LanguageBundle::new(crate::i18n::Locale::En);

        Self {
            current_path: location.navigation_path(),
            location,
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

    /// Window location used at IO / address-bar boundaries.
    pub fn location(&self) -> &EPath {
        &self.location
    }

    pub fn display_path(&self) -> String {
        self.location.display()
    }

    pub fn internal_display(&self) -> String {
        self.location.internal_display()
    }

    pub fn is_mount(&self) -> bool {
        Mounter::is_mount(&self.location)
    }

    pub fn with_navigation_path(&self, path: PathBuf) -> EPath {
        self.location.with_navigation_path(path)
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
        self.location.parent().is_some()
    }

    pub fn begin_load(&mut self) {
        self.loading = true;
        self.error = None;
        self.status = StatusInfo::Loading;
    }

    /// Validate and begin loading a navigation path in this window.
    pub fn navigate(&mut self, path: PathBuf) -> Option<PathBuf> {
        let target = self.location.with_navigation_path(path.clone());
        if !target.exists() {
            self.error = Some(ModelError::InvalidPath);
            self.status = StatusInfo::Path(self.bundle.tr(ids::ERROR_INVALID_PATH));
            return None;
        }

        if !target.is_directory() {
            self.error = Some(ModelError::NotDirectory);
            self.status = StatusInfo::Path(self.bundle.tr(ids::ERROR_NOT_DIRECTORY));
            return None;
        }

        self.begin_load();
        Some(path)
    }

    /// Begin loading a listed directory (skip re-validation; entry came from a listing).
    pub fn navigate_dir(&mut self, dir: DirEntry) -> PathBuf {
        self.begin_load();
        dir.path.clone()
    }

    pub fn go_up(&mut self) -> Option<PathBuf> {
        let parent = self.location.parent()?;
        self.navigate(parent.navigation_path())
    }

    pub fn refresh(&mut self) -> Option<PathBuf> {
        self.begin_load();
        Some(self.current_path.clone())
    }

    pub fn select_entry(&mut self, index: usize) {
        self.selected_index = Some(index);
    }

    pub fn select_path(&mut self, path: &Path) {
        self.selected_index = self.entries.iter().position(|entry| entry.path() == path);
    }

    pub fn open_entry(&mut self, index: usize) -> Option<OpenEntryAction> {
        let entry = self.entries.get(index)?;

        if let Some(dir) = entry.as_dir() {
            let dir = dir.clone();
            self.navigate_dir(dir.clone());
            return Some(OpenEntryAction::Navigate(dir));
        }

        let epath = self
            .location
            .with_navigation_path(entry.path().to_path_buf());
        if let Some(archive) = epath.as_mountable_device() {
            return Some(OpenEntryAction::OpenArchive(archive));
        }

        if preview::is_previewable(entry.fs_entry()) {
            return Some(OpenEntryAction::Preview(entry.fs_entry().clone()));
        }

        match preview::open_with_system(entry.fs_entry()) {
            Ok(()) => {
                self.status = StatusInfo::Opened {
                    name: entry.name().to_string(),
                };
                Some(OpenEntryAction::OpenedSystem {
                    name: entry.name().to_string(),
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
        result: Result<(PathBuf, Vec<FileEntry>), String>,
    ) {
        self.loading = false;
        match result {
            Ok((path, entries)) => {
                self.location = self.location.with_navigation_path(path.clone());
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
