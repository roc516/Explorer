use std::path::{Component, Path, PathBuf};

use explorer_core::filesystem::{disk_path, host_backend, DeviceId, EPath, Mounter};

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
    let disk_backend = host_backend().id();

    let mut segments = device_id_breadcrumbs(container, disk_backend, path.backend());
    let mut acc = Mounter::mount_path(container.clone(), PathBuf::new(), path.backend());

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

fn device_id_breadcrumbs(
    id: &DeviceId,
    disk_backend: &'static str,
    archive_backend: &'static str,
) -> Vec<PathBreadcrumb> {
    match id {
        DeviceId::Host(path) => disk_breadcrumbs(path, disk_backend),
        DeviceId::Nested { parent, entry } => {
            let mut segments = device_id_breadcrumbs(parent, disk_backend, archive_backend);
            let mut acc = Mounter::mount_path((**parent).clone(), PathBuf::new(), archive_backend);

            // Parent archive root crumb already ends at parent; append entry components
            // as paths inside the parent mount.
            for component in entry.components() {
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
    }
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
