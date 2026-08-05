use crate::command::Command;
use crate::diff::{DiffEntry, TreeDiff, TreeDiffer};
use crate::domain::commit::Commit;
use crate::domain::hash::Hash;
use crate::error::VctrlError;
use crate::storage::traits::{ObjectStore, ObjectStoreExt, RefStore};

#[derive(Debug, Clone)]
pub struct ShowOutput {
    pub commit: Commit,
    pub diff: Option<Vec<DiffEntry>>,
}

pub struct Show {
    pub commit_hash: Hash,
}

impl Command for Show {
    type Output = ShowOutput;

    fn execute(
        &self,
        store: &mut dyn ObjectStore,
        _refs: &mut dyn RefStore,
    ) -> Result<ShowOutput, VctrlError> {
        let commit = store.get_commit(&self.commit_hash)?;
        let diff = if let Some(parent_hash) = commit.parents.first().copied() {
            let parent_commit = store.get_commit(&parent_hash)?;
            let parent_tree = store.get_tree(&parent_commit.tree)?;
            let commit_tree = store.get_tree(&commit.tree)?;
            let differ = TreeDiffer;
            Some(differ.diff(&parent_tree, &commit_tree)?)
        } else {
            None
        };

        Ok(ShowOutput { commit, diff })
    }
}
