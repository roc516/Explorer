use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use explorer_core::BlockDevice;
use explorer_core::filesystem::{Mounter, MountedFs, try_host};
use explorer_core::{DirEntry, FsEntry};

use crate::entry::mount_root_dir;

#[derive(Debug, Clone)]
pub struct TreeNode {
    entry: Arc<dyn DirEntry>,
}

impl TreeNode {
    pub fn from_dir(entry: Arc<dyn DirEntry>) -> Self {
        Self { entry }
    }

    pub fn name(&self) -> &str {
        self.entry.name()
    }

    pub fn path(&self) -> &Path {
        self.entry.path()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeRow {
    pub path: PathBuf,
    pub name: String,
    pub depth: usize,
    pub expanded: bool,
    pub loading: bool,
    pub selected: bool,
    pub expandable: bool,
}

pub struct DirectoryTree {
    roots: Vec<TreeNode>,
    expanded: BTreeSet<PathBuf>,
    children: HashMap<PathBuf, Vec<TreeNode>>,
    loading: BTreeSet<PathBuf>,
    selected: Option<PathBuf>,
}

impl DirectoryTree {
    pub fn new() -> Self {
        let root = if cfg!(windows) {
            PathBuf::from("C:\\")
        } else {
            PathBuf::from("/")
        };
        let host = try_host().expect("host backend not registered");
        let mounted = host
            .mount(&root)
            .unwrap_or_else(|message| panic!("cannot mount root: {message}"));
        let mount: Arc<dyn MountedFs> = Arc::from(mounted);
        Self::with_roots(vec![TreeNode::from_dir(mount_root_dir(mount))])
    }

    pub fn for_mounted(device: BlockDevice) -> Self {
        let mount = Mounter::mount(device).unwrap_or_else(|message| {
            panic!("unsupported archive: {message}")
        });
        Self::with_roots(vec![TreeNode::from_dir(mount_root_dir(mount))])
    }

    fn with_roots(roots: Vec<TreeNode>) -> Self {
        Self {
            roots,
            expanded: BTreeSet::new(),
            children: HashMap::new(),
            loading: BTreeSet::new(),
            selected: None,
        }
    }

    pub fn rows(&self) -> Vec<TreeRow> {
        let mut rows = Vec::new();
        self.append_rows(&self.roots, 0, &mut rows);
        rows
    }

    /// Expand/collapse. When expand needs a load, returns the directory to `list`.
    pub fn toggle(&mut self, path: PathBuf) -> Option<Arc<dyn DirEntry>> {
        if self.expanded.contains(&path) {
            self.expanded.remove(&path);
            return None;
        }

        self.expanded.insert(path.clone());
        if self.children.contains_key(&path) {
            None
        } else {
            self.begin_load(&path)
        }
    }

    pub fn select(&mut self, path: PathBuf) -> Option<Arc<dyn DirEntry>> {
        self.selected = Some(path.clone());
        self.find_entry(&path).cloned()
    }

    pub fn on_children_loaded(
        &mut self,
        path: PathBuf,
        result: Result<Vec<TreeNode>, String>,
    ) -> Option<Arc<dyn DirEntry>> {
        self.loading.remove(&path);

        match result {
            Ok(children) => {
                self.children.insert(path, children);
            }
            Err(_) => {
                self.expanded.remove(&path);
            }
        }

        self.next_pending_load()
    }

    /// Drop cached listings and reload expanded folders (and host roots when applicable).
    pub fn refresh(&mut self) -> Option<Arc<dyn DirEntry>> {
        self.children.clear();
        self.loading.clear();
        self.next_pending_load()
    }

    /// Mark ancestors expanded and return the next directory that still needs listing.
    pub fn sync_selection(&mut self, current: &Path) -> Option<Arc<dyn DirEntry>> {
        self.selected = Some(current.to_path_buf());
        self.next_sync_load(current)
    }

    fn next_sync_load(&mut self, current: &Path) -> Option<Arc<dyn DirEntry>> {
        for path in ancestors_and_self(current) {
            self.expanded.insert(path.clone());
            if self.children.contains_key(&path) || self.loading.contains(&path) {
                continue;
            }
            return self.begin_load(&path);
        }
        None
    }

    /// Next directory to list: toward selection first, then other expanded folders.
    fn next_pending_load(&mut self) -> Option<Arc<dyn DirEntry>> {
        if let Some(selected) = self.selected.clone() {
            if let Some(entry) = self.next_sync_load(selected.as_path()) {
                return Some(entry);
            }
        }

        let mut candidates: Vec<_> = self.expanded.iter().cloned().collect();
        candidates.sort_by_key(|path| path.components().count());
        for path in candidates {
            if self.children.contains_key(&path) || self.loading.contains(&path) {
                continue;
            }
            if let Some(entry) = self.begin_load(&path) {
                return Some(entry);
            }
        }
        None
    }

    fn begin_load(&mut self, path: &Path) -> Option<Arc<dyn DirEntry>> {
        let entry = self.find_entry(path)?.clone();
        self.loading.insert(path.to_path_buf());
        Some(entry)
    }

    fn find_entry(&self, path: &Path) -> Option<&Arc<dyn DirEntry>> {
        self.roots
            .iter()
            .chain(self.children.values().flatten())
            .find(|node| node.path() == path)
            .map(|node| &node.entry)
    }

    fn append_rows(&self, nodes: &[TreeNode], depth: usize, rows: &mut Vec<TreeRow>) {
        for node in nodes {
            let path = node.path().to_path_buf();
            let expanded = self.expanded.contains(&path);
            rows.push(TreeRow {
                path: path.clone(),
                name: node.name().to_string(),
                depth,
                expanded,
                loading: self.loading.contains(&path),
                selected: self.selected.as_ref() == Some(&path),
                expandable: self.is_expandable(&path),
            });

            if expanded {
                if let Some(children) = self.children.get(&path) {
                    self.append_rows(children, depth + 1, rows);
                }
            }
        }
    }

    fn is_expandable(&self, path: &Path) -> bool {
        if self.loading.contains(path) {
            return true;
        }

        match self.children.get(path) {
            Some(children) => !children.is_empty(),
            None => true,
        }
    }
}

impl Default for DirectoryTree {
    fn default() -> Self {
        Self::new()
    }
}

/// List directory children for the tree via the retained [`DirEntry`] handle.
pub fn load_tree_children(dir: &Arc<dyn DirEntry>) -> Result<Vec<TreeNode>, String> {
    Ok(dir
        .list()?
        .into_iter()
        .filter_map(|entry| match entry {
            FsEntry::Dir(d) => Some(TreeNode::from_dir(d)),
            FsEntry::File(_) => None,
            FsEntry::Volume(_) => None,
        })
        .collect())
}

fn ancestors_and_self(path: &Path) -> Vec<PathBuf> {
    let mut chain = Vec::new();
    let mut current = path.to_path_buf();
    loop {
        chain.push(current.clone());
        match current.parent() {
            Some(parent) if parent != current.as_path() => {
                current = parent.to_path_buf();
            }
            _ => break,
        }
    }
    chain.reverse();
    chain
}
