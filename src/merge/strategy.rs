use crate::domain::hash::Hash;
use crate::error::VctrlError;
use crate::storage::traits::ObjectStore;

pub enum MergeResult {
    Success(Hash),
    Conflict(String),
}

pub trait MergeStrategy {
    fn merge(
        &self,
        store: &mut dyn ObjectStore,
        base: &Hash,
        ours: &Hash,
        theirs: &Hash,
    ) -> Result<MergeResult, VctrlError>;
}
