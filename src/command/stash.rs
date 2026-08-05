use crate::codec::Encoder;
use crate::command::Command;
use crate::domain::commit::Commit;
use crate::domain::hash::Hash;
use crate::domain::object::Object;
use crate::domain::user::UserID;
use crate::error::VctrlError;
use crate::hashing::Hasher;
use crate::storage::traits::{ObjectStore, ObjectStoreExt, RefStore};

pub struct StashPush {
    pub tree_hash: Hash,
    pub author: UserID,
    pub message: Option<String>,
    pub encoder: Box<dyn Encoder>,
    pub hasher: Box<dyn Hasher>,
}

impl Command for StashPush {
    type Output = Hash;

    fn execute(
        &self,
        store: &mut dyn ObjectStore,
        refs: &mut dyn RefStore,
    ) -> Result<Hash, VctrlError> {
        let msg = self.message.clone().unwrap_or_else(|| "stash".to_string());
        let commit = Commit::new(
            self.tree_hash,
            vec![],
            self.author.clone(),
            self.author.clone(),
            msg,
            None,
        );
        let mut buf = Vec::new();
        self.encoder.encode_commit(&commit, &mut buf)?;
        let hash = self.hasher.hash_commit_encoded(&buf);
        store.put(&hash, &Object::Commit(Box::new(commit)))?;

        let existing = refs.list_refs("refs/stash/")?;
        let next_index = existing.len();
        let ref_name = format!("refs/stash/{}", next_index);
        refs.set_ref(&ref_name, &hash)?;
        Ok(hash)
    }
}

pub struct StashPop;

impl Command for StashPop {
    type Output = Option<Hash>;

    fn execute(
        &self,
        store: &mut dyn ObjectStore,
        refs: &mut dyn RefStore,
    ) -> Result<Option<Hash>, VctrlError> {
        let stash_refs = refs.list_refs("refs/stash/")?;
        if stash_refs.is_empty() {
            return Ok(None);
        }
        let mut indices: Vec<usize> = stash_refs
            .iter()
            .filter_map(|r| r.trim_start_matches("refs/stash/").parse::<usize>().ok())
            .collect();
        indices.sort();
        let last = indices.last().unwrap();
        let ref_name = format!("refs/stash/{}", last);
        let commit_hash = refs.get_ref(&ref_name)?.unwrap();
        let commit = store.get_commit(&commit_hash)?;
        refs.delete_ref(&ref_name)?;
        Ok(Some(commit.tree))
    }
}

pub struct StashList;

impl Command for StashList {
    type Output = Vec<(String, Hash)>;

    fn execute(
        &self,
        _store: &mut dyn ObjectStore,
        refs: &mut dyn RefStore,
    ) -> Result<Vec<(String, Hash)>, VctrlError> {
        let stash_refs = refs.list_refs("refs/stash/")?;
        let mut result = Vec::new();
        for r in stash_refs {
            if let Some(hash) = refs.get_ref(&r)? {
                result.push((r, hash));
            }
        }
        Ok(result)
    }
}
