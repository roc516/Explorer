use std::path::{Path, PathBuf};

use super::builders::{disk_breadcrumbs, PathBreadcrumb};
use super::epath::{disk_path, EPath};
use super::mounter::Mounter;
use crate::filesystem::EntryKind;

impl EPath {
    pub fn parent(&self) -> Option<EPath> {
        if Mounter::is_mount(self) {
            let (container, inner) = Mounter::mount_ref(self).ok()?;
            if inner.as_os_str().is_empty() {
                return None;
            }
            let parent = inner.parent().unwrap_or(Path::new(""));
            Some(Mounter::mount_path(
                container.to_path_buf(),
                parent.to_path_buf(),
                self.backend,
            ))
        } else {
            let disk = self.disk_ref().ok()?;
            disk.parent()
                .map(|parent| disk_path(parent.to_path_buf(), self.backend))
        }
    }

    pub fn join_dir(&self, name: &str) -> EPath {
        if Mounter::is_mount(self) {
            let (container, inner) =
                Mounter::mount_ref(self).unwrap_or((Path::new(""), Path::new("")));
            let inner = if inner.as_os_str().is_empty() {
                PathBuf::from(name)
            } else {
                inner.join(name)
            };
            Mounter::mount_path(container.to_path_buf(), inner, self.backend)
        } else {
            let disk = self.disk_ref().unwrap_or(Path::new(""));
            disk_path(disk.join(name), self.backend)
        }
    }

    pub fn display(&self) -> String {
        if Mounter::is_mount(self) {
            let (container, inner) =
                Mounter::mount_ref(self).unwrap_or((Path::new(""), Path::new("")));
            if inner.as_os_str().is_empty() {
                container.display().to_string()
            } else {
                format!("{}\\{}", container.display(), inner.display())
            }
        } else {
            self.disk_ref()
                .map(|disk| disk.display().to_string())
                .unwrap_or_default()
        }
    }

    pub fn breadcrumbs(&self) -> Vec<PathBreadcrumb> {
        if Mounter::is_mount(self) {
            self.mount_breadcrumbs()
        } else {
            self.disk_ref()
                .map(|disk| disk_breadcrumbs(disk, self.backend))
                .unwrap_or_default()
        }
    }

    fn mount_breadcrumbs(&self) -> Vec<PathBreadcrumb> {
        let (container, inner) = match Mounter::mount_ref(self) {
            Ok(parts) => parts,
            Err(_) => return Vec::new(),
        };
        let disk_backend = crate::filesystem::backends::try_registry()
            .and_then(|registry| registry.disk_backend())
            .map(|backend| backend.id());
        let Some(disk_backend) = disk_backend else {
            return Vec::new();
        };

        let mut segments = disk_breadcrumbs(container, disk_backend);
        let mut acc = Mounter::mount_path(container.to_path_buf(), PathBuf::new(), self.backend);

        for component in inner.components() {
            if let std::path::Component::Normal(name) = component {
                acc = acc.join_dir(name.to_str().unwrap_or_default());
                segments.push(PathBreadcrumb {
                    path: acc.clone(),
                    label: name.to_string_lossy().into_owned(),
                });
            }
        }

        segments
    }

    pub fn exists(&self) -> bool {
        if let Ok(backend) = self.resolve() {
            backend.exists(self)
        } else {
            false
        }
    }

    pub fn is_file(&self) -> bool {
        if let Ok(disk) = self.disk_ref() {
            return disk.is_file();
        }
        let Ok(backend) = self.resolve() else { return false };
        let Ok((container, inner)) = Mounter::mount_ref(self) else { return false };
        matches!(backend.kind(container, inner), Some(EntryKind::File))
    }

    pub fn is_directory(&self) -> bool {
        if let Ok(disk) = self.disk_ref() {
            return disk.is_dir();
        }
        let Ok(backend) = self.resolve() else { return false };
        let Ok((container, inner)) = Mounter::mount_ref(self) else { return false };
        matches!(backend.kind(container, inner), Some(EntryKind::Directory))
    }

    pub fn file_name(&self) -> String {
        self.path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default()
    }

    pub fn extension(&self) -> Option<String> {
        self.path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(str::to_ascii_lowercase)
    }

    pub(crate) fn archive_container(&self) -> Option<&std::path::Path> {
        Mounter::mount_ref(self).ok().map(|(container, _)| container)
    }

    pub fn nested_archive_file(&self) -> Option<PathBuf> {
        let disk = self.disk_ref().ok()?;
        if disk.is_file() && crate::filesystem::backends::is_mounted_path(disk) {
            Some(disk.to_path_buf())
        } else {
            None
        }
    }

    pub fn open_with_system(&self) -> Result<(), String> {
        let path = if Mounter::is_mount(self) {
            let temp_dir = std::env::temp_dir().join("explorer-archive-preview");
            std::fs::create_dir_all(&temp_dir).map_err(|err| err.to_string())?;
            let file_name = {
                let name = self.file_name();
                if name.is_empty() {
                    "preview.bin".to_string()
                } else {
                    name
                }
            };
            let output = temp_dir.join(file_name);
            let backend = self.resolve()?;
            std::fs::write(&output, backend.read(self)?)
                .map_err(|err| err.to_string())?;
            output
        } else {
            self.disk_ref()?.to_path_buf()
        };
        open::that(&path).map_err(|err| err.to_string())
    }
}
