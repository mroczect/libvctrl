use crate::codec::Encoder;
use crate::command::Command;
use crate::domain::hash::Hash;
use crate::error::VctrlError;
use crate::hashing::Hasher;
use crate::merge::{ConflictResolver, ThreeWayMerge};
use crate::storage::traits::{ObjectStore, RefStore};
pub struct MergeCommand {
    pub base: Hash,
    pub ours: Hash,
    pub theirs: Hash,
    pub merger: Box<dyn ThreeWayMerge>,
    pub resolver: Box<dyn ConflictResolver>,
    pub encoder: Box<dyn Encoder>,
    pub hasher: Box<dyn Hasher>,
}
impl Command for MergeCommand {
    type Output = Hash;
    fn execute(
        &self,
        store: &mut dyn ObjectStore,
        _refs: &mut dyn RefStore,
    ) -> Result<Hash, VctrlError> {
        self.merger.merge(
            store,
            &self.base,
            &self.ours,
            &self.theirs,
            self.resolver.as_ref(),
            self.encoder.as_ref(),
            self.hasher.as_ref(),
        )
    }
}
