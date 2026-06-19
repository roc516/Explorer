use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use crate::fs::{list_drives, read_directory};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeNode {
    pub name: String,
    pub path: PathBuf,
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
            .map(|path| TreeNode {
                name: path.display().to_string(),
                path,
            })
            .collect();

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

    pub fn toggle(&mut self, path: PathBuf) -> Option<PathBuf> {
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

    pub fn select(&mut self, path: PathBuf) {
        self.selected = Some(path);
    }

    pub fn on_children_loaded(&mut self, path: PathBuf, result: Result<Vec<TreeNode>, String>) {
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

    pub fn sync_selection(&mut self, current: &Path) -> Vec<PathBuf> {
        self.selected = Some(current.to_path_buf());

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

pub fn load_tree_children(path: &Path) -> Result<Vec<TreeNode>, String> {
    Ok(read_directory(path)?
        .into_iter()
        .filter(|entry| entry.is_dir)
        .map(|entry| TreeNode {
            name: entry.name,
            path: entry.path,
        })
        .collect())
}

fn ancestors_and_self(path: &Path) -> Vec<PathBuf> {
    let mut chain = Vec::new();
    let mut current = Some(path);

    while let Some(part) = current {
        chain.push(part.to_path_buf());
        current = part.parent();
    }

    chain.reverse();
    chain
}
