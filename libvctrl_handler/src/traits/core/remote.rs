use crate::errors::VctrlError;

/// Trait for interacting with remote repositories.
pub trait Remote: Send + Sync {
    /// The refspec type.
    type RefSpec: Send + Sync;

    /// The remote reference type.
    type RemoteRef: Send + Sync;

    /// Lists references available on the remote.
    fn list_refs(&self) -> Result<Vec<Self::RemoteRef>, VctrlError>;

    /// Fetches objects according to the given refspecs.
    fn fetch(&mut self, refspecs: &[Self::RefSpec]) -> Result<(), VctrlError>;

    /// Pushes objects according to the given refspecs.
    fn push(&mut self, refspecs: &[Self::RefSpec]) -> Result<(), VctrlError>;
}
