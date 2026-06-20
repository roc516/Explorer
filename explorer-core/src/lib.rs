mod entry;
pub mod filesystem;
mod i18n;
mod model;
mod navigation;
mod preview;
mod tree;

pub use entry::FileEntry;
pub use filesystem::{
    ensure_backends_registered, path_breadcrumbs, read_directory, PathBreadcrumb, PathOps,
};
pub use i18n::{detect_system_locale, ids, Language, LanguageBundle, Locale};
pub use model::{ExplorerModel, ModelError, OpenEntryAction, StatusInfo};
pub use navigation::NavigationHistory;
pub use preview::{
    load_preview, ImagePreview, PdfPreview, PptPreview, PreviewFile, PreviewKind, TextEncoding,
    TextPreview, WordPreview,
};
pub use tree::{load_tree_children, DirectoryTree, TreeNode, TreeRow};
