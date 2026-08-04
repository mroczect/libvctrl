use crate::handler::error::VctrlError;
use crate::handler::types::{Hash, RefStore};

fn validate_branch(name: &str) -> Result<(), VctrlError> {
    if !name.starts_with("refs/heads/") {
        Err(VctrlError::InvalidRef(
            "branch name must start with 'refs/heads/'".into(),
        ))
    } else {
        Ok(())
    }
}

pub fn create_branch(
    ref_store: &mut dyn RefStore,
    name: &str,
    commit_hash: &Hash,
) -> Result<(), VctrlError> {
    validate_branch(name)?;
    ref_store.set_ref(name, commit_hash)
}

pub fn delete_branch(ref_store: &mut dyn RefStore, name: &str) -> Result<(), VctrlError> {
    validate_branch(name)?;
    ref_store.delete_ref(name)
}

pub fn get_branch(ref_store: &dyn RefStore, name: &str) -> Result<Option<Hash>, VctrlError> {
    validate_branch(name)?;
    ref_store.get_ref(name)
}

pub fn set_head_branch(ref_store: &mut dyn RefStore, branch: &str) -> Result<(), VctrlError> {
    validate_branch(branch)?;
    ref_store.set_head(branch)
}
