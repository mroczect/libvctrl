use crate::codec::Encoder;
use crate::command::Command;
use crate::diff::{DiffEntry, TreeDiff, TreeDiffer};
use crate::domain::hash::Hash;
use crate::error::VctrlError;
use crate::hashing::Hasher;
use crate::patch::{apply_patch, generate_patch};
use crate::storage::traits::{ObjectStore, ObjectStoreExt, RefStore};

pub struct DiffCommits {
    pub old_commit: Hash,
    pub new_commit: Hash,
}

impl Command for DiffCommits {
    type Output = Vec<DiffEntry>;

    fn execute(
        &self,
        store: &mut dyn ObjectStore,
        _refs: &mut dyn RefStore,
    ) -> Result<Vec<DiffEntry>, VctrlError> {
        let old_tree_hash = store.get_commit(&self.old_commit)?.tree;
        let new_tree_hash = store.get_commit(&self.new_commit)?.tree;

        let old_tree = store.get_tree(&old_tree_hash)?;
        let new_tree = store.get_tree(&new_tree_hash)?;

        let differ = TreeDiffer;
        differ.diff(&old_tree, &new_tree)
    }
}

pub struct DiffPatch {
    pub old_tree_hash: Hash,
    pub new_tree_hash: Hash,
}

impl Command for DiffPatch {
    type Output = Vec<u8>;

    fn execute(
        &self,
        store: &mut dyn ObjectStore,
        _refs: &mut dyn RefStore,
    ) -> Result<Vec<u8>, VctrlError> {
        let old_tree = store.get_tree(&self.old_tree_hash)?;
        let new_tree = store.get_tree(&self.new_tree_hash)?;
        generate_patch(&old_tree, &new_tree)
    }
}

pub struct ApplyPatch {
    pub base_tree_hash: Hash,
    pub patch_data: Vec<u8>,
    pub encoder: Box<dyn Encoder>,
    pub hasher: Box<dyn Hasher>,
}

impl Command for ApplyPatch {
    type Output = Hash;

    fn execute(
        &self,
        store: &mut dyn ObjectStore,
        _refs: &mut dyn RefStore,
    ) -> Result<Hash, VctrlError> {
        let base_tree = store.get_tree(&self.base_tree_hash)?;
        let new_tree = apply_patch(&base_tree, &self.patch_data, store, self.hasher.as_ref())?;
        let mut buf = Vec::new();
        self.encoder.encode_tree(&new_tree, &mut buf)?;
        let hash = self.hasher.hash_tree_encoded(&buf);
        store.put(&hash, &crate::domain::object::Object::Tree(new_tree))?;
        Ok(hash)
    }
}
