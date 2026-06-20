use std::path::{Component, PathBuf};

use super::ops::EPath;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathBreadcrumb {
    pub path: EPath,
    pub label: String,
}

pub fn disk_breadcrumbs(path: &std::path::Path, backend: &'static str) -> Vec<PathBreadcrumb> {
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
                    last.path = super::ops::disk_path(acc.clone(), backend);
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
        push_disk_breadcrumb(&mut segments, &path.to_path_buf(), backend, path.display().to_string());
    }

    segments
}

fn push_disk_breadcrumb(
    segments: &mut Vec<PathBreadcrumb>,
    path: &std::path::Path,
    backend: &'static str,
    label: String,
) {
    segments.push(PathBreadcrumb {
        path: super::ops::disk_path(path.to_path_buf(), backend),
        label,
    });
}
