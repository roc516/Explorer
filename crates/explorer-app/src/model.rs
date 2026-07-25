use std::path::{Path, PathBuf};

use explorer_core::filesystem::{
    is_mountable, navigation_parent, BlockDevice, Mounter, EPath,
};
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
    /// Current directory handle (listable).
    current_dir: DirEntry,
    /// Archive window identity (`path` stays at mount root). `None` for disk windows.
    mount: Option<EPath>,
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
        let current_dir = DirEntry::open_host(initial).unwrap_or_else(|message| {
            panic!("cannot open home directory: {message}")
        });
        Self::with_dir(current_dir, None)
    }

    pub fn new_mounted(device: BlockDevice) -> Self {
        let (root, current_dir) = Mounter::mount_root_dir(device).unwrap_or_else(|message| {
            panic!("unsupported archive: {message}")
        });
        Self::with_dir(current_dir, Some(root))
    }

    fn with_dir(current_dir: DirEntry, mount: Option<EPath>) -> Self {
        let bundle = LanguageBundle::new(crate::i18n::Locale::En);

        Self {
            current_dir,
            mount,
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

    pub fn current_dir(&self) -> &DirEntry {
        &self.current_dir
    }

    pub fn current_path(&self) -> &Path {
        &self.current_dir.path
    }

    /// Rebuild a full [`EPath`] for address-bar / mount helpers.
    pub fn location(&self) -> EPath {
        match &self.mount {
            Some(root) => root.with_navigation_path(self.current_dir.path.clone()),
            None => EPath::local(self.current_dir.path.clone()),
        }
    }

    pub fn display_path(&self) -> String {
        self.location().display()
    }

    pub fn internal_display(&self) -> String {
        self.location().internal_display()
    }

    pub fn is_mount(&self) -> bool {
        self.mount.is_some()
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
        navigation_parent(self.current_path(), self.is_mount()).is_some()
    }

    pub fn begin_load(&mut self) {
        self.loading = true;
        self.error = None;
        self.status = StatusInfo::Loading;
    }

    /// Resolve a navigation path to a [`DirEntry`] in this window.
    pub fn resolve_dir(&self, path: PathBuf) -> Result<DirEntry, ModelError> {
        if let Some(root) = &self.mount {
            let target = root.with_navigation_path(path.clone());
            if !target.exists() {
                return Err(ModelError::InvalidPath);
            }
            if !target.is_directory() {
                return Err(ModelError::NotDirectory);
            }
            Mounter::dir_at(root, &path).map_err(ModelError::External)
        } else {
            if !path.exists() {
                return Err(ModelError::InvalidPath);
            }
            if !path.is_dir() {
                return Err(ModelError::NotDirectory);
            }
            DirEntry::open_host(path).map_err(ModelError::External)
        }
    }

    /// Validate and begin loading a navigation path in this window.
    pub fn navigate(&mut self, path: PathBuf) -> Option<DirEntry> {
        match self.resolve_dir(path) {
            Ok(dir) => {
                self.begin_load();
                Some(dir)
            }
            Err(error) => {
                self.set_path_error(error);
                None
            }
        }
    }

    /// Begin loading a listed directory (skip re-validation; entry came from a listing).
    pub fn navigate_dir(&mut self, dir: DirEntry) -> DirEntry {
        self.begin_load();
        dir
    }

    pub fn go_up(&mut self) -> Option<DirEntry> {
        let parent = navigation_parent(self.current_path(), self.is_mount())?;
        self.navigate(parent)
    }

    pub fn refresh(&mut self) -> Option<DirEntry> {
        self.begin_load();
        Some(self.current_dir.clone())
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

        if let Some(archive) = self.as_mountable(entry) {
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

    fn as_mountable(&self, entry: &FileEntry) -> Option<BlockDevice> {
        let file = entry.as_file()?;
        if let Some(root) = &self.mount {
            root.with_navigation_path(file.path.clone())
                .as_mountable_device()
        } else {
            let device = BlockDevice::open_host(file.path.clone()).ok()?;
            is_mountable(&device).then_some(device)
        }
    }

    pub fn on_directory_loaded(
        &mut self,
        result: Result<(DirEntry, Vec<FileEntry>), String>,
    ) {
        self.loading = false;
        match result {
            Ok((dir, entries)) => {
                self.current_dir = dir;
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
