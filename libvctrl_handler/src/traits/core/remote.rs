use crate::errors::VctrlError;

pub trait Remote: Send + Sync {
    type RefSpec: Send + Sync;
    type RemoteRef: Send + Sync;

    fn list_refs(&self) -> Result<Vec<Self::RemoteRef>, VctrlError>;
    fn fetch(&mut self, refspecs: &[Self::RefSpec]) -> Result<(), VctrlError>;
    fn push(&mut self, refspecs: &[Self::RefSpec]) -> Result<(), VctrlError>;
}
