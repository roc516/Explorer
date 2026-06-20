use crate::filesystem::backends::try_registry;

use super::ops::{mount_path, PathOps};

pub(crate) fn from_address_input(input: &str, context: &PathOps) -> PathOps {
    let trimmed = input.trim();
    if let Some(container) = context.archive_container() {
        let prefix = format!("{}\\", container.display());
        let inner = trimmed
            .strip_prefix(&prefix)
            .or_else(|| trimmed.strip_prefix(&container.display().to_string()))
            .unwrap_or(trimmed);
        let backend = context.mount_backend().or_else(|| {
            try_registry()
                .and_then(|registry| registry.find_backend(container))
                .map(|backend| backend.id())
        });
        if let Some(backend) = backend {
            return mount_path(
                container.to_path_buf(),
                normalize_inner_path(inner),
                backend,
            );
        }
    }

    PathOps::local(trimmed)
}

pub(crate) fn parent_path(path: &PathOps) -> Option<PathOps> {
    path.parent()
}

fn normalize_inner_path(value: &str) -> std::path::PathBuf {
    use std::path::Component;

    let mut result = std::path::PathBuf::new();
    for component in std::path::Path::new(value).components() {
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
