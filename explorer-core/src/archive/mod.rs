use std::collections::BTreeSet;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::browse_path::BrowsePath;
use crate::entry::FileEntry;

pub fn read_directory(path: &BrowsePath) -> Result<Vec<FileEntry>, String> {
    match path {
        BrowsePath::Local(local) => crate::fs::read_local_directory(local),
        BrowsePath::Archive { file, inner } => read_zip_directory(file, inner),
    }
}

pub fn read_zip_directory(archive: &Path, inner: &Path) -> Result<Vec<FileEntry>, String> {
    let file = File::open(archive).map_err(|err| err.to_string())?;
    let mut zip = zip::ZipArchive::new(file).map_err(|err| err.to_string())?;
    let prefix = zip_prefix(inner);

    let mut directories = BTreeSet::new();
    let mut files = Vec::new();

    for index in 0..zip.len() {
        let entry = zip.by_index(index).map_err(|err| err.to_string())?;
        let name = entry.name().replace('\\', "/");
        if name.ends_with('/') {
            continue;
        }

        let Some(relative) = strip_prefix(&name, &prefix) else {
            continue;
        };

        if relative.is_empty() {
            continue;
        }

        let parts: Vec<&str> = relative.split('/').collect();
        if parts.len() == 1 {
            files.push(FileEntry {
                name: parts[0].to_string(),
                path: BrowsePath::Archive {
                    file: archive.to_path_buf(),
                    inner: join_inner(inner, parts[0]),
                },
                is_dir: false,
                size: entry.size(),
                modified: None,
            });
        } else {
            directories.insert(parts[0].to_string());
        }
    }

    let mut items = directories
        .into_iter()
        .map(|name| FileEntry {
            path: BrowsePath::Archive {
                file: archive.to_path_buf(),
                inner: join_inner(inner, &name),
            },
            name,
            is_dir: true,
            size: 0,
            modified: None,
        })
        .collect::<Vec<_>>();

    items.append(&mut files);
    items.sort_by(|left, right| match (left.is_dir, right.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => left.name.to_lowercase().cmp(&right.name.to_lowercase()),
    });

    Ok(items)
}

pub fn path_exists(path: &BrowsePath) -> bool {
    match path {
        BrowsePath::Local(local) => local.exists(),
        BrowsePath::Archive { file, inner } => {
            if !file.is_file() {
                return false;
            }
            if inner.as_os_str().is_empty() {
                return true;
            }
            entry_kind(file, inner).is_some()
        }
    }
}

pub fn is_file_entry(path: &BrowsePath) -> bool {
    matches!(entry_kind_path(path), Some(ZipEntryKind::File))
}

pub fn is_dir_entry(path: &BrowsePath) -> bool {
    matches!(entry_kind_path(path), Some(ZipEntryKind::Directory))
}

fn entry_kind_path(path: &BrowsePath) -> Option<ZipEntryKind> {
    let BrowsePath::Archive { file, inner } = path else {
        return None;
    };
    if inner.as_os_str().is_empty() {
        return Some(ZipEntryKind::Directory);
    }
    entry_kind(file, inner)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ZipEntryKind {
    File,
    Directory,
}

fn entry_kind(archive: &Path, inner: &Path) -> Option<ZipEntryKind> {
    let zip_file = File::open(archive).ok()?;
    let mut zip = zip::ZipArchive::new(zip_file).ok()?;
    let name = zip_entry_name(inner);

    if zip.by_name(&name).is_ok() {
        return Some(ZipEntryKind::File);
    }

    let prefix = if name.is_empty() {
        String::new()
    } else {
        format!("{name}/")
    };

    for index in 0..zip.len() {
        let entry = zip.by_index(index).ok()?;
        let entry_name = entry.name().replace('\\', "/");
        if prefix.is_empty() {
            if !entry_name.is_empty() {
                return Some(ZipEntryKind::Directory);
            }
            continue;
        }
        if entry_name.starts_with(&prefix) {
            return Some(ZipEntryKind::Directory);
        }
    }

    None
}

pub fn with_entry_reader<R>(
    path: &BrowsePath,
    mut f: impl FnMut(&mut dyn Read, u64) -> Result<R, String>,
) -> Result<R, String> {
    let BrowsePath::Archive { file, inner } = path else {
        return Err("archive-entry-required".to_string());
    };

    if inner.as_os_str().is_empty() {
        return Err("archive-entry-required".to_string());
    }

    let zip_file = File::open(file).map_err(|err| err.to_string())?;
    let mut zip = zip::ZipArchive::new(zip_file).map_err(|err| err.to_string())?;
    let entry_name = zip_entry_name(inner);
    let mut entry = zip.by_name(&entry_name).map_err(|err| err.to_string())?;
    let size = entry.size();

    f(&mut entry, size)
}

pub fn extract_entry_to_temp(path: &BrowsePath) -> Result<PathBuf, String> {
    let BrowsePath::Archive { inner, .. } = path else {
        return Err("archive-entry-required".to_string());
    };

    let temp_dir = std::env::temp_dir().join("explorer-archive-preview");
    std::fs::create_dir_all(&temp_dir).map_err(|err| err.to_string())?;
    let file_name = inner
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "preview.bin".to_string());
    let output = temp_dir.join(file_name);

    with_entry_reader(path, |reader, _size| {
        let mut output_file = File::create(&output).map_err(|err| err.to_string())?;
        std::io::copy(reader, &mut output_file).map_err(|err| err.to_string())?;
        Ok(())
    })?;

    Ok(output)
}

fn zip_prefix(inner: &Path) -> String {
    let mut parts = Vec::new();
    for component in inner.components() {
        if let std::path::Component::Normal(name) = component {
            parts.push(name.to_string_lossy().into_owned());
        }
    }

    if parts.is_empty() {
        String::new()
    } else {
        format!("{}/", parts.join("/"))
    }
}

fn strip_prefix<'a>(name: &'a str, prefix: &str) -> Option<&'a str> {
    if prefix.is_empty() {
        return Some(name);
    }

    name.strip_prefix(prefix)
}

fn join_inner(inner: &Path, name: &str) -> PathBuf {
    if inner.as_os_str().is_empty() {
        PathBuf::from(name)
    } else {
        inner.join(name)
    }
}

fn zip_entry_name(inner: &Path) -> String {
    let mut parts = Vec::new();
    for component in inner.components() {
        if let std::path::Component::Normal(name) = component {
            parts.push(name.to_string_lossy().into_owned());
        }
    }
    parts.join("/")
}
