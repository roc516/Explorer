use std::path::PathBuf;

use explorer_core::TreeNode;

#[derive(Debug, Clone)]
pub enum Message {
    Toggle(PathBuf),
    Select(PathBuf),
    ChildrenLoaded(PathBuf, Result<Vec<TreeNode>, String>),
}
