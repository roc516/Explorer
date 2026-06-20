use std::path::{Component, Path, PathBuf};

use explorer_core::filesystem::{disk_breadcrumbs, try_registry, Mounter, PathNavigation, EPath};

use super::identity::ID;
use super::ZipBackend;

impl PathNavigation for ZipBackend {
    fn parent(&self, path: &EPath) -> Option<EPath> {
        let (container, inner) = Mounter::mount_ref(path).ok()?;
        if inner.as_os_str().is_empty() {
            return None;
        }
        let parent = inner.parent().unwrap_or(Path::new(""));
        Some(Mounter::mount_path(
            container.to_path_buf(),
            parent.to_path_buf(),
            ID,
        ))
    }

    fn join_dir(&self, path: &EPath, name: &str) -> EPath {
        let (container, inner) = Mounter::mount_ref(path).unwrap_or((Path::new(""), Path::new("")));
        let inner = if inner.as_os_str().is_empty() {
            PathBuf::from(name)
        } else {
            inner.join(name)
        };
        Mounter::mount_path(container.to_path_buf(), inner, ID)
    }

    fn display(&self, path: &EPath) -> String {
        let (container, inner) = Mounter::mount_ref(path).unwrap_or((Path::new(""), Path::new("")));
        if inner.as_os_str().is_empty() {
            container.display().to_string()
        } else {
            format!("{}\\{}", container.display(), inner.display())
        }
    }

    fn breadcrumbs(&self, path: &EPath) -> Vec<explorer_core::filesystem::PathBreadcrumb> {
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
        let mut acc = Mounter::mount_path(container.to_path_buf(), PathBuf::new(), ID);

        for component in inner.components() {
            if let Component::Normal(name) = component {
                acc = self.join_dir(&acc, name.to_str().unwrap_or_default());
                segments.push(explorer_core::filesystem::PathBreadcrumb {
                    path: acc.clone(),
                    label: name.to_string_lossy().into_owned(),
                });
            }
        }

        segments
    }
}
