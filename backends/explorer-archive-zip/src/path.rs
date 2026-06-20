use std::path::{Component, Path};

pub fn entry_name(inner: &Path) -> String {
    let mut parts = Vec::new();
    for component in inner.components() {
        if let Component::Normal(name) = component {
            parts.push(name.to_string_lossy().into_owned());
        }
    }
    parts.join("/")
}

pub fn zip_prefix(inner: &Path) -> String {
    let mut parts = Vec::new();
    for component in inner.components() {
        if let Component::Normal(name) = component {
            parts.push(name.to_string_lossy().into_owned());
        }
    }

    if parts.is_empty() {
        String::new()
    } else {
        format!("{}/", parts.join("/"))
    }
}

pub fn strip_prefix<'a>(name: &'a str, prefix: &str) -> Option<&'a str> {
    if prefix.is_empty() {
        return Some(name);
    }

    name.strip_prefix(prefix)
}
