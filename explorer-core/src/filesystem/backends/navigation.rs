use crate::filesystem::path::{PathBreadcrumb, PathOps};

pub trait PathNavigation {
    fn parent(&self, path: &PathOps) -> Option<PathOps>;
    fn join_dir(&self, path: &PathOps, name: &str) -> PathOps;
    fn display(&self, path: &PathOps) -> String;
    fn breadcrumbs(&self, path: &PathOps) -> Vec<PathBreadcrumb>;
}
