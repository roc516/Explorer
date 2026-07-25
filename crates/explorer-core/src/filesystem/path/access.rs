use std::path::{Path, PathBuf};

use super::epath::{disk_path, EPath};
use super::mounter::Mounter;
use super::reader::mount_entry_name;
use crate::entry::{FileEntry, FsEntry};
use crate::filesystem::backends::{is_mountable, BlockDevice, DeviceId, EntryKind};

impl EPath {
    pub fn parent(&self) -> Option<EPath> {
        if Mounter::is_mount(self) {
            let (container, inner) = Mounter::mount_ref(self).ok()?;
            if inner.as_os_str().is_empty() {
                return None;
            }
            let parent = inner.parent().unwrap_or(Path::new(""));
            Some(Mounter::mount_path(
                container.clone(),
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
            let Ok((container, inner)) = Mounter::mount_ref(self) else {
                return disk_path(PathBuf::from(name), self.backend);
            };
            let inner = if inner.as_os_str().is_empty() {
                PathBuf::from(name)
            } else {
                inner.join(name)
            };
            Mounter::mount_path(container.clone(), inner, self.backend)
        } else {
            let disk = self.disk_ref().unwrap_or(Path::new(""));
            disk_path(disk.join(name), self.backend)
        }
    }

    pub fn display(&self) -> String {
        if Mounter::is_mount(self) {
            let Ok((container, inner)) = Mounter::mount_ref(self) else {
                return String::new();
            };
            if inner.as_os_str().is_empty() {
                container.display()
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
        self.mount_entry_kind().is_some()
    }

    pub fn is_file(&self) -> bool {
        if let Ok(disk) = self.disk_ref() {
            return disk.is_file();
        }
        matches!(self.mount_entry_kind(), Some(EntryKind::File))
    }

    pub fn is_directory(&self) -> bool {
        if let Ok(disk) = self.disk_ref() {
            return disk.is_dir();
        }
        matches!(self.mount_entry_kind(), Some(EntryKind::Directory))
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

    /// If this path is a mountable archive, build a [`BlockDevice`] for it.
    pub fn as_mountable_device(&self) -> Option<BlockDevice> {
        if !self.is_file() {
            return None;
        }

        let device = if Mounter::is_mount(self) {
            let file = FileEntry::resolve(self).ok()?;
            let data = file.read().ok()?;
            BlockDevice::from_bytes(
                DeviceId::Nested {
                    parent: Box::new(self.root.clone()),
                    entry: self.path.clone(),
                },
                file.name,
                data,
            )
        } else {
            let disk = self.disk_ref().ok()?;
            BlockDevice::open_host(disk.to_path_buf()).ok()?
        };

        is_mountable(&device).then_some(device)
    }

    pub fn open_with_system(&self) -> Result<(), String> {
        let path = if Mounter::is_mount(self) {
            let temp_dir = std::env::temp_dir().join("explorer-archive-preview");
            std::fs::create_dir_all(&temp_dir).map_err(|err| err.to_string())?;
            let file = FileEntry::resolve(self)?;
            let file_name = if file.name.is_empty() {
                "preview.bin".to_string()
            } else {
                file.name.clone()
            };
            let output = temp_dir.join(file_name);
            std::fs::write(&output, file.read()?).map_err(|err| err.to_string())?;
            output
        } else {
            self.disk_ref()?.to_path_buf()
        };
        open::that(&path).map_err(|err| err.to_string())
    }

    fn mount_entry_kind(&self) -> Option<EntryKind> {
        let full = mount_entry_name(&self.path);
        if full.is_empty() {
            return Some(EntryKind::Directory);
        }

        let (parent, child) = match full.rsplit_once('/') {
            Some((parent, child)) => (parent.to_string(), child.to_string()),
            None => (String::new(), full),
        };

        let entries = Mounter::list_at(self, &parent).ok()?;
        entries.into_iter().find_map(|entry| match entry {
            FsEntry::File(file) if file.name == child => Some(EntryKind::File),
            FsEntry::Dir(dir) if dir.name == child => Some(EntryKind::Directory),
            _ => None,
        })
    }
}
