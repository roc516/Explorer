use std::path::{Path, PathBuf};

pub fn file_name_of(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Parent of an in-window navigation path.
///
/// For mounts, the archive root (`""`) has no parent.
pub fn navigation_parent(path: &Path, is_mount: bool) -> Option<PathBuf> {
    if is_mount {
        if path.as_os_str().is_empty() {
            return None;
        }
        Some(
            path.parent()
                .unwrap_or_else(|| Path::new(""))
                .to_path_buf(),
        )
    } else {
        path.parent().map(|parent| parent.to_path_buf())
    }
}
