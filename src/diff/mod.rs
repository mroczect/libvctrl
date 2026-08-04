pub mod tree_diff;
pub use tree_diff::*;

use crate::domain::hash::Hash;
use crate::domain::tree::Tree;
use crate::error::VctrlError;

#[derive(Debug, Clone)]
pub enum DiffKind {
    Added,
    Removed,
    Modified { old_hash: Hash, new_hash: Hash },
}
#[derive(Debug, Clone)]
pub struct DiffEntry {
    pub name: String,
    pub kind: DiffKind,
}
pub trait TreeDiff {
    fn diff(&self, old_tree: &Tree, new_tree: &Tree) -> Result<Vec<DiffEntry>, VctrlError>;
}
