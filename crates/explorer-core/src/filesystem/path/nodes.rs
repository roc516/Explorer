use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Volume {
    pub path: PathBuf,
    pub label: String,
}

impl Volume {
    pub fn new(path: PathBuf, label: String) -> Self {
        Volume { path, label }
    }
}
