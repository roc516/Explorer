use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BrowsePath {
    Local(PathBuf),
    Archive {
        file: PathBuf,
        inner: PathBuf,
    },
}

impl BrowsePath {
    pub fn local(path: impl Into<PathBuf>) -> Self {
        Self::Local(path.into())
    }

    pub fn archive_root(file: PathBuf) -> Self {
        Self::Archive {
            file,
            inner: PathBuf::new(),
        }
    }

    pub fn is_archive_entry(&self) -> bool {
        matches!(self, Self::Archive { .. })
    }

    pub fn local_file(&self) -> Option<&Path> {
        match self {
            Self::Local(path) => Some(path),
            Self::Archive { .. } => None,
        }
    }

    pub fn archive_file(&self) -> Option<&Path> {
        match self {
            Self::Archive { file, .. } => Some(file),
            Self::Local(_) => None,
        }
    }

    pub fn parent(&self) -> Option<Self> {
        match self {
            Self::Local(path) => path.parent().map(|parent| Self::Local(parent.to_path_buf())),
            Self::Archive { file, inner } => {
                if inner.as_os_str().is_empty() {
                    return None;
                }
                let parent = inner.parent().unwrap_or(Path::new(""));
                Some(Self::Archive {
                    file: file.clone(),
                    inner: parent.to_path_buf(),
                })
            }
        }
    }

    pub fn join_dir(&self, name: &str) -> Self {
        match self {
            Self::Local(path) => Self::Local(path.join(name)),
            Self::Archive { file, inner } => Self::Archive {
                file: file.clone(),
                inner: if inner.as_os_str().is_empty() {
                    PathBuf::from(name)
                } else {
                    inner.join(name)
                },
            },
        }
    }

    pub fn display(&self) -> String {
        match self {
            Self::Local(path) => path.display().to_string(),
            Self::Archive { file, inner } => {
                if inner.as_os_str().is_empty() {
                    file.display().to_string()
                } else {
                    format!("{}\\{}", file.display(), inner.display())
                }
            }
        }
    }

    pub fn from_address_input(input: &str, archive_file: Option<&Path>) -> Self {
        let trimmed = input.trim();
        if let Some(archive) = archive_file {
            let prefix = format!("{}\\", archive.display());
            let inner = trimmed
                .strip_prefix(&prefix)
                .or_else(|| trimmed.strip_prefix(&archive.display().to_string()))
                .unwrap_or(trimmed);
            return Self::Archive {
                file: archive.to_path_buf(),
                inner: normalize_inner_path(inner),
            };
        }

        Self::Local(PathBuf::from(trimmed))
    }

    pub fn exists(&self) -> bool {
        crate::archive::path_exists(self)
    }

    pub fn is_file(&self) -> bool {
        match self {
            Self::Local(path) => path.is_file(),
            Self::Archive { .. } => crate::archive::is_file_entry(self),
        }
    }

    pub fn is_directory(&self) -> bool {
        match self {
            Self::Local(path) => path.is_dir(),
            Self::Archive { .. } => crate::archive::is_dir_entry(self),
        }
    }

    pub fn file_name(&self) -> String {
        match self {
            Self::Local(path) => path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default(),
            Self::Archive { inner, .. } => inner
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default(),
        }
    }

    pub fn extension(&self) -> Option<String> {
        let path = match self {
            Self::Local(path) => path.as_path(),
            Self::Archive { inner, .. } => inner.as_path(),
        };
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(str::to_ascii_lowercase)
    }

    pub fn preview_path(&self) -> PathBuf {
        match self {
            Self::Local(path) => path.clone(),
            Self::Archive { file, inner } => file.join(inner),
        }
    }
}

fn normalize_inner_path(value: &str) -> PathBuf {
    let mut result = PathBuf::new();
    for component in Path::new(value).components() {
        match component {
            Component::Normal(name) => result.push(name),
            Component::ParentDir => {
                result.pop();
            }
            _ => {}
        }
    }
    result
}

pub fn is_archive_extension(ext: &str) -> bool {
    matches!(
        ext,
        "zip" | "7z" | "rar" | "tar" | "gz" | "tgz" | "bz2" | "xz" | "jar" | "apk"
    )
}

pub fn is_archive_path(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| is_archive_extension(&ext.to_ascii_lowercase()))
        .unwrap_or(false)
}
