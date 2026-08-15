use crate::VctrlError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Change {
    Added,

    Deleted,

    Modified,
}

pub trait TreeDiffer {
    type TreeId;

    type Path;

    fn diff_trees(&self, old: &Self::TreeId, new: &Self::TreeId)
    -> Result<Vec<Change>, VctrlError>;
}
