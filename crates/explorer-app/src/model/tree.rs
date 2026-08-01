use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use explorer_core::{DirEntry, FsEntry};

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

/// 业务数据：目录树结构（根节点 + 已加载子目录）。
/// 属于 `ExplorerState`，UI 组件通过参数借用。
#[derive(Clone)]
pub struct TreeState {
    roots: Vec<TreeNode>,
    children: HashMap<PathBuf, Vec<TreeNode>>,
}

impl TreeState {
    pub fn new(roots: Vec<TreeNode>) -> Self {
        Self {
            roots,
            children: HashMap::new(),
        }
    }

    pub fn roots(&self) -> &[TreeNode] {
        &self.roots
    }

    pub fn children_of(&self, path: &Path) -> Option<&[TreeNode]> {
        self.children.get(path).map(Vec::as_slice)
    }

    pub fn has_children(&self, path: &Path) -> bool {
        self.children.contains_key(path)
    }

    pub fn insert_children(&mut self, path: PathBuf, children: Vec<TreeNode>) {
        self.children.insert(path, children);
    }

    pub fn clear_children(&mut self) {
        self.children.clear();
    }

    pub fn find_entry(&self, path: &Path) -> Option<&Arc<dyn DirEntry>> {
        self.roots
            .iter()
            .chain(self.children.values().flatten())
            .find(|node| node.path() == path)
            .map(|node| &node.entry)
    }
}

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
