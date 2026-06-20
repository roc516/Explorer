use crate::filesystem::path::{PathBreadcrumb, EPath};

pub trait PathNavigation {
    fn parent(&self, path: &EPath) -> Option<EPath>;
    fn join_dir(&self, path: &EPath, name: &str) -> EPath;
    fn display(&self, path: &EPath) -> String;
    fn breadcrumbs(&self, path: &EPath) -> Vec<PathBreadcrumb>;
}
