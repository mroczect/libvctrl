use crate::VctrlError;

pub trait RevWalk {
    type CommitId;

    fn parents(&self, id: &Self::CommitId) -> Result<Vec<Self::CommitId>, VctrlError>;
}
