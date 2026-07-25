use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use explorer_core::filesystem::{list_drives, BlockDevice, Mounter, MountedDevice};
use explorer_core::{DirEntry, Directory, FsEntry};

#[derive(Debug, Clone)]
pub struct TreeNode {
    entry: DirEntry,
}

impl TreeNode {
    pub fn from_dir(entry: DirEntry) -> Self {
        Self { entry }
    }

    pub fn name(&self) -> &str {
        &self.entry.name
    }

    pub fn path(&self) -> &Path {
        &self.entry.path
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
        let roots = list_drives()
            .into_iter()
            .map(TreeNode::from_dir)
            .collect();

        Self::with_roots(roots)
    }

    pub fn for_mounted(device: BlockDevice) -> Self {
        let name = device.name().to_string();
        let name = if name.is_empty() {
            device.id().display()
        } else {
            name
        };

        let root = Mounter::mount_root(device).unwrap_or_else(|message| {
            panic!("unsupported archive: {message}")
        });
        let mounted = Mounter::device(&root).unwrap_or_else(|message| {
            panic!("mount device unavailable: {message}")
        });

        Self::with_roots(vec![TreeNode::from_dir(DirEntry::new(
            name,
            PathBuf::new(),
            Arc::new(MountedRoot(mounted)),
        ))])
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
    pub fn toggle(&mut self, path: PathBuf) -> Option<DirEntry> {
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

    pub fn select(&mut self, path: PathBuf) {
        self.selected = Some(path);
    }

    pub fn on_children_loaded(
        &mut self,
        path: PathBuf,
        result: Result<Vec<TreeNode>, String>,
    ) -> Option<DirEntry> {
        self.loading.remove(&path);

        match result {
            Ok(children) => {
                self.children.insert(path, children);
            }
            Err(_) => {
                self.expanded.remove(&path);
                return None;
            }
        }

        // Continue expanding toward the selected path after a parent finishes loading.
        self.selected
            .clone()
            .and_then(|selected| self.next_sync_load(selected.as_path()))
    }

    /// Mark ancestors expanded and return the next directory that still needs listing.
    pub fn sync_selection(&mut self, current: &Path) -> Option<DirEntry> {
        self.selected = Some(current.to_path_buf());
        self.next_sync_load(current)
    }

    fn next_sync_load(&mut self, current: &Path) -> Option<DirEntry> {
        for path in ancestors_and_self(current) {
            self.expanded.insert(path.clone());
            if self.children.contains_key(&path) || self.loading.contains(&path) {
                continue;
            }
            return self.begin_load(&path);
        }
        None
    }

    fn begin_load(&mut self, path: &Path) -> Option<DirEntry> {
        let entry = self.find_entry(path)?.clone();
        self.loading.insert(path.to_path_buf());
        Some(entry)
    }

    fn find_entry(&self, path: &Path) -> Option<&DirEntry> {
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
        if self.loading.contains(path) || self.expanded.contains(path) {
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
pub fn load_tree_children(dir: &DirEntry) -> Result<Vec<TreeNode>, String> {
    Ok(dir
        .list()?
        .into_iter()
        .filter_map(|entry| match entry {
            FsEntry::Dir(d) => Some(TreeNode::from_dir(d)),
            FsEntry::File(_) => None,
        })
        .collect())
}

struct MountedRoot(Arc<dyn MountedDevice>);

impl Directory for MountedRoot {
    fn list(&self) -> Result<Vec<FsEntry>, String> {
        self.0.list()
    }
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
