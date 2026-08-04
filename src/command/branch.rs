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
        _store: &mut dyn ObjectStore,
        refs: &mut dyn RefStore,
    ) -> Result<(), VctrlError> {
        if !(self.target.starts_with("refs/") || Hash::from_hex(&self.target).is_ok()) {
            return Err(VctrlError::InvalidRef(
                "HEAD target must be a symbolic ref (refs/...) or a 128-char hex hash".into(),
            ));
        }
        refs.set_head(&self.target)
    }
}