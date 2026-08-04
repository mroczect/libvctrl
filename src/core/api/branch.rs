use crate::handler::error::VctrlError;
use crate::handler::types::{Hash, RefStore};

pub fn create_branch(
    ref_store: &mut dyn RefStore,
    name: &str,
    commit_hash: &Hash,
) -> Result<(), VctrlError> {
    if !name.starts_with("refs/heads/") {
        return Err(VctrlError::InvalidRef(
            "branch name must start with 'refs/heads/'".into(),
        ));
    }
    ref_store.set_ref(name, commit_hash)
}

pub fn delete_branch(ref_store: &mut dyn RefStore, name: &str) -> Result<(), VctrlError> {
    if !name.starts_with("refs/heads/") {
        return Err(VctrlError::InvalidRef(
            "branch name must start with 'refs/heads/'".into(),
        ));
    }
    ref_store.delete_ref(name)
}

pub fn get_branch(ref_store: &dyn RefStore, name: &str) -> Result<Option<Hash>, VctrlError> {
    if !name.starts_with("refs/heads/") {
        return Err(VctrlError::InvalidRef(
            "branch name must start with 'refs/heads/'".into(),
        ));
    }
    ref_store.get_ref(name)
}

pub fn set_head_branch(ref_store: &mut dyn RefStore, branch: &str) -> Result<(), VctrlError> {
    if !branch.starts_with("refs/heads/") {
        return Err(VctrlError::InvalidRef(
            "branch name must start with 'refs/heads/'".into(),
        ));
    }
    ref_store.set_head(branch)
}
