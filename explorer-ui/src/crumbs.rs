use std::path::{Component, Path, PathBuf};

use explorer_core::filesystem::{disk_path, try_registry, EPath, Mounter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathBreadcrumb {
    pub path: EPath,
    pub label: String,
}

/// Build breadcrumbs for an EPath, handling both disk and mount paths.
pub fn breadcrumbs(path: &EPath) -> Vec<PathBreadcrumb> {
    if Mounter::is_mount(path) {
        mount_breadcrumbs(path)
    } else {
        path.disk_ref()
            .map(|disk| disk_breadcrumbs(disk, path.backend()))
            .unwrap_or_default()
    }
}

fn mount_breadcrumbs(path: &EPath) -> Vec<PathBreadcrumb> {
    let (container, inner) = match Mounter::mount_ref(path) {
        Ok(parts) => parts,
        Err(_) => return Vec::new(),
    };
    let disk_backend = try_registry()
        .and_then(|registry| registry.disk_backend())
        .map(|backend| backend.id());
    let Some(disk_backend) = disk_backend else {
        return Vec::new();
    };

    let mut segments = disk_breadcrumbs(container, disk_backend);
    let mut acc = Mounter::mount_path(container.to_path_buf(), PathBuf::new(), path.backend());

    for component in inner.components() {
        if let Component::Normal(name) = component {
            acc = acc.join_dir(name.to_str().unwrap_or_default());
            segments.push(PathBreadcrumb {
                path: acc.clone(),
                label: name.to_string_lossy().into_owned(),
            });
        }
    }

    segments
}

fn disk_breadcrumbs(path: &Path, backend: &'static str) -> Vec<PathBreadcrumb> {
    let mut segments = Vec::new();
    let mut acc = PathBuf::new();

    for component in path.components() {
        match component {
            Component::Prefix(_) => {
                acc.push(component);
                push_disk_breadcrumb(&mut segments, &acc, backend, acc.display().to_string());
            }
            Component::RootDir => {
                acc.push(component);
                if let Some(last) = segments.last_mut() {
                    last.path = disk_path(acc.clone(), backend);
                } else {
                    push_disk_breadcrumb(&mut segments, &acc, backend, acc.display().to_string());
                }
            }
            Component::Normal(name) => {
                acc.push(component);
                push_disk_breadcrumb(
                    &mut segments,
                    &acc,
                    backend,
                    name.to_string_lossy().into_owned(),
                );
            }
            Component::CurDir | Component::ParentDir => {
                acc.push(component);
            }
        }
    }

    if segments.is_empty() {
        push_disk_breadcrumb(&mut segments, path, backend, path.display().to_string());
    }

    segments
}

fn push_disk_breadcrumb(
    segments: &mut Vec<PathBreadcrumb>,
    path: &Path,
    backend: &'static str,
    label: String,
) {
    segments.push(PathBreadcrumb {
        path: disk_path(path.to_path_buf(), backend),
        label,
    });
}
