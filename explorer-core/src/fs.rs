use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::browse_path::BrowsePath;
use crate::entry::FileEntry;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathBreadcrumb {
    pub path: BrowsePath,
    pub label: String,
}

pub fn path_breadcrumbs(path: &BrowsePath) -> Vec<PathBreadcrumb> {
    match path {
        BrowsePath::Local(local) => local_path_breadcrumbs(local),
        BrowsePath::Archive { file, inner } => {
            let mut segments = local_path_breadcrumbs(file);
            let mut acc = BrowsePath::archive_root(file.clone());

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
    }
}

fn local_path_breadcrumbs(path: &Path) -> Vec<PathBreadcrumb> {
    let mut segments = Vec::new();
    let mut acc = PathBuf::new();

    for component in path.components() {
        match component {
            Component::Prefix(_) => {
                acc.push(component);
                segments.push(PathBreadcrumb {
                    path: BrowsePath::local(acc.clone()),
                    label: acc.display().to_string(),
                });
            }
            Component::RootDir => {
                acc.push(component);
                if let Some(last) = segments.last_mut() {
                    last.path = BrowsePath::local(acc.clone());
                } else {
                    segments.push(PathBreadcrumb {
                        path: BrowsePath::local(acc.clone()),
                        label: acc.display().to_string(),
                    });
                }
            }
            Component::Normal(name) => {
                acc.push(component);
                segments.push(PathBreadcrumb {
                    path: BrowsePath::local(acc.clone()),
                    label: name.to_string_lossy().into_owned(),
                });
            }
            Component::CurDir | Component::ParentDir => {
                acc.push(component);
            }
        }
    }

    if segments.is_empty() {
        segments.push(PathBreadcrumb {
            path: BrowsePath::local(path.to_path_buf()),
            label: path.display().to_string(),
        });
    }

    segments
}

pub fn read_local_directory(path: &Path) -> Result<Vec<FileEntry>, String> {
    let entries = fs::read_dir(path).map_err(|err| err.to_string())?;

    let mut items = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|err| err.to_string())?;
        let metadata = entry.metadata().map_err(|err| err.to_string())?;
        let file_type = entry.file_type().map_err(|err| err.to_string())?;

        items.push(FileEntry {
            name: entry.file_name().to_string_lossy().into_owned(),
            path: BrowsePath::local(entry.path()),
            is_dir: file_type.is_dir(),
            size: if file_type.is_dir() { 0 } else { metadata.len() },
            modified: metadata.modified().ok(),
        });
    }

    items.sort_by(|left, right| match (left.is_dir, right.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => left.name.to_lowercase().cmp(&right.name.to_lowercase()),
    });

    Ok(items)
}

pub fn list_drives() -> Vec<PathBuf> {
    #[cfg(windows)]
    {
        (b'A'..=b'Z')
            .filter_map(|letter| {
                let drive = format!("{}:\\", letter as char);
                let path = PathBuf::from(&drive);
                if path.exists() {
                    Some(path)
                } else {
                    None
                }
            })
            .collect()
    }

    #[cfg(not(windows))]
    {
        vec![PathBuf::from("/")]
    }
}

pub fn default_initial_path() -> PathBuf {
    dirs::document_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| PathBuf::from("C:\\"))
}

pub fn parent_path(path: &BrowsePath) -> Option<BrowsePath> {
    path.parent()
}

pub fn open_with_system(path: &Path) -> Result<(), String> {
    open::that(path).map_err(|err| err.to_string())
}
