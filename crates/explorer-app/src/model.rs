use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use explorer_core::BlockDevice;
use explorer_core::filesystem::{
    entry_at, is_mountable, navigation_parent, MountedFs, Mounter, try_host,
};
use explorer_core::{DirEntry, FsEntry};

use crate::entry::{mount_root_dir, FileEntry};
use crate::i18n::{ids, LanguageBundle};
use crate::preview;

pub mod file_list;
pub mod tree;
pub use file_list::FileListState;
pub use tree::{load_tree_children, TreeState, TreeNode, TreeRow};

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
    Navigate(Arc<dyn DirEntry>),
    Preview(explorer_core::FsEntry),
    OpenArchive(BlockDevice),
    OpenedSystem { name: String },
}

#[derive(Debug, Clone)]
pub enum AddressTarget {
    Directory(Arc<dyn DirEntry>),
    File { path: PathBuf },
}

#[derive(Clone)]
pub struct ExplorerState {
    current_dir: Arc<dyn DirEntry>,
    mount: Arc<dyn MountedFs>,
    pub file_list: FileListState,
    pub bundle: LanguageBundle,
}

impl ExplorerState {
    pub fn new_local() -> Self {
        let initial = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"))
            });
        let root = if cfg!(windows) {
            PathBuf::from("C:\\")
        } else {
            PathBuf::from("/")
        };
        let host = try_host().expect("host backend not registered");
        let mounted = host
            .mount(&root)
            .unwrap_or_else(|message| panic!("cannot mount root: {message}"));
        let mount: Arc<dyn MountedFs> = Arc::from(mounted);
        let current_dir = match entry_at(mount.as_ref(), &initial) {
            Ok(FsEntry::Dir(dir)) => dir,
            _ => mount_root_dir(mount.clone()),
        };
        Self::with_dir(current_dir, mount)
    }

    pub fn new_mounted(device: BlockDevice) -> Self {
        let mount = Mounter::mount(device).unwrap_or_else(|message| {
            panic!("unsupported archive: {message}")
        });
        let current_dir = mount_root_dir(mount.clone());
        Self::with_dir(current_dir, mount)
    }

    fn with_dir(current_dir: Arc<dyn DirEntry>, mount: Arc<dyn MountedFs>) -> Self {
        let bundle = LanguageBundle::new(crate::i18n::Locale::En);

        Self {
            current_dir,
            mount,
            file_list: FileListState::new(),
            bundle,
        }
    }

    pub fn new() -> Self {
        Self::new_local()
    }

    pub fn current_dir(&self) -> &Arc<dyn DirEntry> {
        &self.current_dir
    }

    pub fn current_path(&self) -> &Path {
        self.current_dir.path()
    }

    pub fn mount(&self) -> &Arc<dyn MountedFs> {
        &self.mount
    }

    pub fn display_path(&self) -> String {
        let p = self.current_path().display().to_string();
        if p.is_empty() { "/".to_string() } else { p }
    }

    pub fn internal_display(&self) -> String {
        self.display_path()
    }

    pub fn set_locale(&mut self, locale: crate::i18n::Locale) {
        self.bundle = LanguageBundle::new(locale);
    }

    pub fn status_text(&self) -> String {
        self.file_list.status_text(&self.bundle)
    }

    pub fn error_text(&self) -> Option<String> {
        self.file_list.error_text(&self.bundle)
    }

    pub fn can_go_up(&self) -> bool {
        navigation_parent(self.current_path(), true).is_some()
    }

    pub fn begin_load(&mut self) {
        self.file_list.begin_load();
    }

    pub fn parse_address(&self, input: &str) -> PathBuf {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return PathBuf::new();
        }
        let path = PathBuf::from(trimmed);
        if path.has_root() { path } else { self.current_path().join(path) }
    }

    pub fn resolve_address(&self, input: &str) -> Result<AddressTarget, ModelError> {
        let path = self.parse_address(input);
        let mount = &self.mount;
        if path.as_os_str().is_empty() {
            return Ok(AddressTarget::Directory(mount_root_dir(mount.clone())));
        }
        match entry_at(mount.as_ref(), &path) {
            Ok(FsEntry::Dir(dir)) => Ok(AddressTarget::Directory(dir)),
            Ok(FsEntry::File(file)) => Ok(AddressTarget::File { path: file.path().to_path_buf() }),
            Ok(FsEntry::Volume(_)) => Err(ModelError::InvalidPath),
            Err(_) => Err(ModelError::InvalidPath),
        }
    }

    pub fn resolve_dir(&self, path: PathBuf) -> Result<Arc<dyn DirEntry>, ModelError> {
        let mount = &self.mount;
        if path.as_os_str().is_empty() {
            return Ok(mount_root_dir(mount.clone()));
        }
        match entry_at(mount.as_ref(), &path) {
            Ok(FsEntry::Dir(dir)) => Ok(dir),
            Ok(FsEntry::File(_)) => Err(ModelError::NotDirectory),
            Ok(FsEntry::Volume(_)) => Err(ModelError::NotDirectory),
            Err(_) => Err(ModelError::InvalidPath),
        }
    }

    pub fn navigate(&mut self, path: PathBuf) -> Option<Arc<dyn DirEntry>> {
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

    pub fn navigate_dir(&mut self, dir: Arc<dyn DirEntry>) -> Arc<dyn DirEntry> {
        self.begin_load();
        dir
    }

    pub fn go_up(&mut self) -> Option<Arc<dyn DirEntry>> {
        let parent = navigation_parent(self.current_path(), true)?;
        self.navigate(parent)
    }

    pub fn refresh(&mut self) -> Option<Arc<dyn DirEntry>> {
        self.begin_load();
        Some(self.current_dir.clone())
    }

    pub fn select_entry(&mut self, index: usize) {
        self.file_list.select_entry(index);
    }

    pub fn select_path(&mut self, path: &Path) {
        self.file_list.select_path(path);
    }

    pub fn open_entry(&mut self, index: usize) -> Option<OpenEntryAction> {
        let entry = self.file_list.entries.get(index)?;

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
                self.file_list.status = StatusInfo::Opened {
                    name: entry.name().to_string(),
                };
                Some(OpenEntryAction::OpenedSystem {
                    name: entry.name().to_string(),
                })
            }
            Err(message) => {
                self.file_list.error = Some(ModelError::External(message));
                self.file_list.status = StatusInfo::LoadFailed;
                None
            }
        }
    }

    fn as_mountable(&self, entry: &FileEntry) -> Option<BlockDevice> {
        let file = entry.as_file()?;
        let mut reader = file.open().ok()?;
        let mut data = Vec::new();
        reader.read_to_end(&mut data).ok()?;
        let device = BlockDevice::from_bytes(file.name().to_string(), data);
        is_mountable(&device).then_some(device)
    }

    pub fn on_directory_loaded(
        &mut self,
        result: Result<(Arc<dyn DirEntry>, Vec<FileEntry>), String>,
    ) {
        if let Ok((ref dir, _)) = result {
            self.current_dir = dir.clone();
        }
        self.file_list.on_directory_loaded(result);
    }

    pub fn set_path_error(&mut self, error: ModelError) {
        let message = match &error {
            ModelError::InvalidPath => self.bundle.tr(ids::ERROR_INVALID_PATH),
            ModelError::NotDirectory => self.bundle.tr(ids::ERROR_NOT_DIRECTORY),
            ModelError::External(message) => message.clone(),
        };
        self.file_list.error = Some(error);
        self.file_list.status = StatusInfo::Path(message);
    }
}