mod archive;
mod browse_path;
mod entry;
mod fs;
mod i18n;
mod model;
mod navigation;
mod preview;
mod tree;

pub use browse_path::{is_archive_path, BrowsePath};
pub use entry::FileEntry;
pub use fs::{
    default_initial_path, list_drives, open_with_system, parent_path, path_breadcrumbs,
    read_local_directory, PathBreadcrumb,
};
pub use i18n::{detect_system_locale, ids, Language, LanguageBundle, Locale};
pub use model::{ExplorerModel, ModelError, OpenEntryAction, StatusInfo, WindowKind};
pub use navigation::NavigationHistory;
pub use preview::{
    load_preview, ImagePreview, PdfPreview, PptPreview, PreviewFile, PreviewKind, TextEncoding,
    TextPreview, WordPreview,
};
pub use tree::{load_tree_children, DirectoryTree, TreeNode, TreeRow};

pub use archive::extract_entry_to_temp;

pub fn read_directory(path: &BrowsePath) -> Result<Vec<FileEntry>, String> {
    archive::read_directory(path)
}
