use std::path::Path;

use explorer_core::filesystem::{disk_breadcrumbs, disk_path, PathNavigation, EPath};

use super::identity::ID;
use super::LocalBackend;

impl PathNavigation for LocalBackend {
    fn parent(&self, path: &EPath) -> Option<EPath> {
        path.disk_ref()
            .ok()?
            .parent()
            .map(|parent| disk_path(parent.to_path_buf(), ID))
    }

    fn join_dir(&self, path: &EPath, name: &str) -> EPath {
        let disk = path.disk_ref().unwrap_or(Path::new(""));
        disk_path(disk.join(name), ID)
    }

    fn display(&self, path: &EPath) -> String {
        path.disk_ref()
            .map(|disk| disk.display().to_string())
            .unwrap_or_default()
    }

    fn breadcrumbs(&self, path: &EPath) -> Vec<explorer_core::filesystem::PathBreadcrumb> {
        path.disk_ref()
            .map(|disk| disk_breadcrumbs(disk, ID))
            .unwrap_or_default()
    }
}