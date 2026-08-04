use crate::codec::Encoder;
use crate::command::Command;
use crate::domain::commit::Commit;
use crate::domain::hash::Hash;
use crate::domain::object::Object;
use crate::domain::user::UserID;
use crate::error::VctrlError;
use crate::hashing::Hasher;
use crate::merge::{ConflictResolver, ThreeWayMerge};
use crate::storage::traits::{ObjectStore, RefStore};

pub struct CherryPick {
    pub commit_hash: Hash,
    pub author: UserID,
    pub committer: UserID,
    pub merger: Box<dyn ThreeWayMerge>,
    pub resolver: Box<dyn ConflictResolver>,
    pub encoder: Box<dyn Encoder>,
    pub hasher: Box<dyn Hasher>,
}

impl Command for CherryPick {
    type Output = Hash;

    fn execute(
        &self,
        store: &mut dyn ObjectStore,
        refs: &mut dyn RefStore,
    ) -> Result<Hash, VctrlError> {
        let src_commit = get_commit(store, &self.commit_hash)?;

        if src_commit.parents.len() > 1 {
            return Err(VctrlError::Other(
                "cherry-pick: merge commits are not supported".into(),
            ));
        }

        let theirs_tree_hash = src_commit.tree;

        let base_tree_hash = match src_commit.parents.first() {
            Some(parent_hash) => {
                let parent_commit = get_commit(store, parent_hash)?;
                parent_commit.tree
            }
            None => {
                let empty_tree =
                    crate::domain::tree::Tree::new(vec![]).map_err(VctrlError::Tree)?;
                let mut buf = Vec::new();
                self.encoder.encode_tree(&empty_tree, &mut buf)?;
                let hash = self.hasher.hash_tree_encoded(&buf);
                store.put(&hash, &Object::Tree(empty_tree))?;
                hash
            }
        };

        let head_commit_hash = refs
            .head()?
            .ok_or_else(|| VctrlError::Other("no HEAD".into()))?;
        let head_commit = get_commit(store, &head_commit_hash)?;
        let ours_tree_hash = head_commit.tree;

        let merged_tree_hash = self.merger.merge(
            store,
            &base_tree_hash,
            &ours_tree_hash,
            &theirs_tree_hash,
            self.resolver.as_ref(),
            self.encoder.as_ref(),
            self.hasher.as_ref(),
        )?;

        let new_commit = Commit::new(
            merged_tree_hash,
            vec![head_commit_hash],
            self.author.clone(),
            self.committer.clone(),
            format!("cherry-pick: {}", &src_commit.message),
            None,
        );
        let mut buf = Vec::new();
        self.encoder.encode_commit(&new_commit, &mut buf)?;
        let new_hash = self.hasher.hash_commit_encoded(&buf);
        store.put(&new_hash, &Object::Commit(Box::new(new_commit)))?;

        if let Some(branch_name) = refs.head_ref_name()? {
            refs.set_ref(&branch_name, &new_hash)?;
        }

        Ok(new_hash)
    }
}

fn get_commit(store: &dyn ObjectStore, hash: &Hash) -> Result<Commit, VctrlError> {
    match store.get(hash)? {
        Some(Object::Commit(c)) => Ok(*c),
        _ => Err(VctrlError::NotFound("commit not found".into())),
    }
}
