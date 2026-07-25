use std::path::{Component, Path, PathBuf};

use explorer_core::filesystem::{DeviceId, EPath, Mounter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathBreadcrumb {
    /// In-window navigation path: absolute disk path, or path relative to the mount root.
    pub path: PathBuf,
    pub label: String,
}

/// Build breadcrumbs for an EPath.
///
/// Mount paths only include segments inside the archive (no host container crumbs).
pub fn breadcrumbs(path: &EPath) -> Vec<PathBreadcrumb> {
    if Mounter::is_mount(path) {
        mount_breadcrumbs(path)
    } else {
        path.disk_ref()
            .map(disk_breadcrumbs)
            .unwrap_or_default()
    }
}

fn mount_breadcrumbs(path: &EPath) -> Vec<PathBreadcrumb> {
    let (container, inner) = match Mounter::mount_ref(path) {
        Ok(parts) => parts,
        Err(_) => return Vec::new(),
    };

    let mut segments = vec![PathBreadcrumb {
        path: PathBuf::new(),
        label: mount_root_label(container),
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

fn mount_root_label(container: &DeviceId) -> String {
    match container {
        DeviceId::Host(path) => path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| path.display().to_string()),
        DeviceId::Nested { entry, .. } => entry
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| entry.display().to_string()),
    }
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
