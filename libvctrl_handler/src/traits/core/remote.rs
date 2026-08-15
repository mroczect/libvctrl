use crate::VctrlError;

pub trait Remote {
    type RefSpec;

    type RemoteRef;

    fn list_refs(&self) -> Result<Vec<Self::RemoteRef>, VctrlError>;

    fn fetch(&mut self, refspecs: &[Self::RefSpec]) -> Result<(), VctrlError>;

    fn push(&mut self, refspecs: &[Self::RefSpec]) -> Result<(), VctrlError>;
}
