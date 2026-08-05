use crate::command::Command;
use crate::domain::hash::Hash;
use crate::error::VctrlError;
use crate::storage::traits::{ObjectStore, RefStore};

fn validate(name: &str) -> Result<(), VctrlError> {
    if !name.starts_with("refs/heads/") {
        Err(VctrlError::InvalidRef(
            "branch name must start with refs/heads/".into(),
        ))
    } else {
        Ok(())
    }
}

pub struct CreateBranch {
    pub name: String,
    pub hash: Hash,
}
impl Command for CreateBranch {
    type Output = ();
    fn execute(
        &self,
        _store: &mut dyn ObjectStore,
        refs: &mut dyn RefStore,
    ) -> Result<(), VctrlError> {
        validate(&self.name)?;
        refs.set_ref(&self.name, &self.hash)
    }
}

pub struct DeleteBranch {
    pub name: String,
}
impl Command for DeleteBranch {
    type Output = ();
    fn execute(
        &self,
        _store: &mut dyn ObjectStore,
        refs: &mut dyn RefStore,
    ) -> Result<(), VctrlError> {
        validate(&self.name)?;
        refs.delete_ref(&self.name)
    }
}

pub struct GetBranch {
    pub name: String,
}
impl Command for GetBranch {
    type Output = Option<Hash>;
    fn execute(
        &self,
        _store: &mut dyn ObjectStore,
        refs: &mut dyn RefStore,
    ) -> Result<Option<Hash>, VctrlError> {
        validate(&self.name)?;
        refs.get_ref(&self.name)
    }
}

pub struct SetHead {
    pub target: String,
}
impl Command for SetHead {
    type Output = ();
    fn execute(
        &self,
        store: &mut dyn ObjectStore,
        refs: &mut dyn RefStore,
    ) -> Result<(), VctrlError> {
        if self.target.starts_with("refs/heads/") {
            if refs.get_ref(&self.target)?.is_none() {
                return Err(VctrlError::InvalidRef(format!(
                    "branch '{}' does not exist",
                    self.target
                )));
            }
            refs.set_head(&self.target)
        } else {
            let hash = Hash::from_hex(&self.target).map_err(|_| {
                VctrlError::InvalidRef("HEAD target must be a branch name or valid hash".into())
            })?;
            if !store.exists(&hash)? {
                return Err(VctrlError::NotFound(format!(
                    "commit '{}' does not exist",
                    self.target
                )));
            }
            refs.set_head(&self.target)
        }
    }
}

pub struct ListBranches;

impl Command for ListBranches {
    type Output = Vec<(String, Hash, bool)>;

    fn execute(
        &self,
        _store: &mut dyn ObjectStore,
        refs: &mut dyn RefStore,
    ) -> Result<Vec<(String, Hash, bool)>, VctrlError> {
        let active_branch = refs.head_ref_name()?;
        let branch_refs = refs.list_refs("refs/heads/")?;
        let mut result = Vec::new();
        for ref_name in branch_refs {
            if let Some(hash) = refs.get_ref(&ref_name)? {
                let short_name = ref_name.trim_start_matches("refs/heads/").to_string();
                let is_active = Some(ref_name.clone()) == active_branch;
                result.push((short_name, hash, is_active));
            }
        }
        Ok(result)
    }
}
