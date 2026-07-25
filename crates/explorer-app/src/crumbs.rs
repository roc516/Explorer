use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathBreadcrumb {
    /// In-window navigation path: absolute disk path, or path relative to the mount root.
    pub path: PathBuf,
    pub label: String,
}

/// Build breadcrumbs for a disk path or a mount-internal path.
///
/// - Disk: pass the absolute path and `root_label = None`.
/// - Mount: pass the path inside the archive and `root_label = Some("/")`.
pub fn breadcrumbs(path: &Path, root_label: Option<String>) -> Vec<PathBreadcrumb> {
    match root_label {
        Some(label) => mount_breadcrumbs(label, path),
        None => disk_breadcrumbs(path),
    }
}

/// Label for the mount-root crumb.
pub fn mount_root_label() -> String {
    "/".to_string()
}

fn mount_breadcrumbs(root_label: String, inner: &Path) -> Vec<PathBreadcrumb> {
    let mut segments = vec![PathBreadcrumb {
        path: PathBuf::new(),
        label: root_label,
    }];
    let mut acc = PathBuf::new();

    for component in inner.components() {
        if let Component::Normal(name) = component {
            acc.push(name);
            segments.push(PathBreadcrumb {
                path: acc.clone(),
                label: name.to_string_lossy().into_owned(),
            });
        }
    }

    segments
}

fn disk_breadcrumbs(path: &Path) -> Vec<PathBreadcrumb> {
    let mut segments = Vec::new();
    let mut acc = PathBuf::new();

    for component in path.components() {
        match component {
            Component::Prefix(_) => {
                acc.push(component);
                push_disk_breadcrumb(&mut segments, &acc, acc.display().to_string());
            }
            Component::RootDir => {
                acc.push(component);
                if let Some(last) = segments.last_mut() {
                    last.path = acc.clone();
                } else {
                    push_disk_breadcrumb(&mut segments, &acc, acc.display().to_string());
                }
            }
            Component::Normal(name) => {
                acc.push(component);
                push_disk_breadcrumb(
                    &mut segments,
                    &acc,
                    name.to_string_lossy().into_owned(),
                );
            }
            Component::CurDir | Component::ParentDir => {
                acc.push(component);
            }
        }
    }

    if segments.is_empty() {
        push_disk_breadcrumb(&mut segments, path, path.display().to_string());
    }

    segments
}

fn push_disk_breadcrumb(segments: &mut Vec<PathBreadcrumb>, path: &Path, label: String) {
    segments.push(PathBreadcrumb {
        path: path.to_path_buf(),
        label,
    });
}
