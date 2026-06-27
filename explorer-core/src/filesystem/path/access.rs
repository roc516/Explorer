use std::path::{Path, PathBuf};

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

    pub fn exists(&self) -> bool {
        if let Ok(disk) = self.disk_ref() {
            return disk.exists();
        }
        Mounter::device(self)
            .map(|device| device.exists(&self.path))
            .unwrap_or(false)
    }

    pub fn is_file(&self) -> bool {
        if let Ok(disk) = self.disk_ref() {
            return disk.is_file();
        }
        Mounter::device(self)
            .map(|device| matches!(device.entry_kind(&self.path), Some(EntryKind::File)))
            .unwrap_or(false)
    }

    pub fn is_directory(&self) -> bool {
        if let Ok(disk) = self.disk_ref() {
            return disk.is_dir();
        }
        Mounter::device(self)
            .map(|device| matches!(device.entry_kind(&self.path), Some(EntryKind::Directory)))
            .unwrap_or(false)
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
            let device = Mounter::device(self)?;
            std::fs::write(&output, device.read(&self.path)?)
                .map_err(|err| err.to_string())?;
            output
        } else {
            self.disk_ref()?.to_path_buf()
        };
        open::that(&path).map_err(|err| err.to_string())
    }
}
