/// Prefix for listing immediate children of directory `name` (`""` = root).
pub fn zip_prefix(name: &str) -> String {
    let name = name.trim_matches(|c| c == '/' || c == '\\');
    if name.is_empty() {
        String::new()
    } else {
        format!("{name}/")
    }
}

pub fn strip_prefix<'a>(name: &'a str, prefix: &str) -> Option<&'a str> {
    if prefix.is_empty() {
        return Some(name);
    }

    name.strip_prefix(prefix)
}

/// Join a mount-relative directory name with a child entry name into a [`PathBuf`].
pub fn join_dir_name(parent: &str, child: &str) -> std::path::PathBuf {
    let mut path = std::path::PathBuf::new();
    for part in parent.split(['/', '\\']) {
        if !part.is_empty() {
            path.push(part);
        }
    }
    path.push(child);
    path
}
