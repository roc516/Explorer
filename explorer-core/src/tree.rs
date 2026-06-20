use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;

use crate::filesystem::{list_drives, read_directory, PathOps};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeNode {
    pub name: String,
    pub path: PathOps,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeRow {
    pub path: PathOps,
    pub name: String,
    pub depth: usize,
    pub expanded: bool,
    pub loading: bool,
    pub selected: bool,
    pub expandable: bool,
}

pub struct DirectoryTree {
    roots: Vec<TreeNode>,
    expanded: BTreeSet<PathOps>,
    children: HashMap<PathOps, Vec<TreeNode>>,
    loading: BTreeSet<PathOps>,
    selected: Option<PathOps>,
}

impl DirectoryTree {
    pub fn new() -> Self {
        let roots = list_drives()
            .into_iter()
            .map(|path| TreeNode {
                name: path.display().to_string(),
                path: PathOps::local(path),
            })
            .collect();

        Self::with_roots(roots)
    }

    pub fn for_mounted(container: PathBuf) -> Self {
        let name = container
            .file_name()
            .map(|value| value.to_string_lossy().into_owned())
            .unwrap_or_else(|| container.display().to_string());

        Self::with_roots(vec![TreeNode {
            name,
            path: PathOps::mount_root(container).unwrap_or_else(|message| {
                panic!("unsupported archive: {message}")
            }),
        }])
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

    pub fn toggle(&mut self, path: PathOps) -> Option<PathOps> {
        if self.expanded.contains(&path) {
            self.expanded.remove(&path);
            return None;
        }

        self.expanded.insert(path.clone());
        if self.children.contains_key(&path) {
            None
        } else {
            self.loading.insert(path.clone());
            Some(path)
        }
    }

    pub fn select(&mut self, path: PathOps) {
        self.selected = Some(path);
    }

    pub fn on_children_loaded(
        &mut self,
        path: PathOps,
        result: Result<Vec<TreeNode>, String>,
    ) {
        self.loading.remove(&path);

        match result {
            Ok(children) => {
                self.children.insert(path, children);
            }
            Err(_) => {
                self.expanded.remove(&path);
            }
        }
    }

    pub fn sync_selection(&mut self, current: &PathOps) -> Vec<PathOps> {
        self.selected = Some(current.clone());

        let mut pending = Vec::new();
        for path in ancestors_and_self(current) {
            self.expanded.insert(path.clone());
            if !self.children.contains_key(&path) && !self.loading.contains(&path) {
                self.loading.insert(path.clone());
                pending.push(path);
            }
        }
        pending
    }

    fn append_rows(&self, nodes: &[TreeNode], depth: usize, rows: &mut Vec<TreeRow>) {
        for node in nodes {
            let expanded = self.expanded.contains(&node.path);
            rows.push(TreeRow {
                path: node.path.clone(),
                name: node.name.clone(),
                depth,
                expanded,
                loading: self.loading.contains(&node.path),
                selected: self.selected.as_ref() == Some(&node.path),
                expandable: self.is_expandable(&node.path),
            });

            if expanded {
                if let Some(children) = self.children.get(&node.path) {
                    self.append_rows(children, depth + 1, rows);
                }
            }
        }
    }

    fn is_expandable(&self, path: &PathOps) -> bool {
        if self.loading.contains(path) || self.expanded.contains(path) {
            return true;
        }

        match self.children.get(path) {
            Some(children) => !children.is_empty(),
            None => true,
        }
    }
}

pub fn load_tree_children(path: &PathOps) -> Result<Vec<TreeNode>, String> {
    Ok(read_directory(path)?
        .into_iter()
        .filter(|entry| entry.is_dir)
        .map(|entry| TreeNode {
            name: entry.name,
            path: entry.path,
        })
        .collect())
}

fn ancestors_and_self(path: &PathOps) -> Vec<PathOps> {
    let mut chain = Vec::new();
    let mut current = Some(path.clone());

    while let Some(part) = current {
        chain.push(part.clone());
        current = part.parent();
    }

    chain.reverse();
    chain
}
