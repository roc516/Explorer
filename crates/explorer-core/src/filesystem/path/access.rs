use super::epath::EPath;
use super::mounter::Mounter;
use super::util::mount_entry_name;
use crate::entry::{FileEntry, FsEntry};
use crate::filesystem::backends::{is_mountable, BlockDevice, DeviceId, EntryKind};

impl EPath {
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
