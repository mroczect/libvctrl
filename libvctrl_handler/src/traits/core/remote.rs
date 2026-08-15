//! Remote repository trait.

use crate::errors::VctrlError;

/// Trait for interacting with remote repositories.
pub trait Remote: Send + Sync {
    /// The refspec type.
    type RefSpec: Send + Sync;

    /// The remote reference type.
    type RemoteRef: Send + Sync;

    /// Lists references available on the remote.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the remote cannot be contacted.
    fn list_refs(&self) -> Result<Vec<Self::RemoteRef>, VctrlError>;

    /// Fetches objects according to the given refspecs.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the fetch fails.
    fn fetch(&mut self, refspecs: &[Self::RefSpec]) -> Result<(), VctrlError>;

    /// Pushes objects according to the given refspecs.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the push fails.
    fn push(&mut self, refspecs: &[Self::RefSpec]) -> Result<(), VctrlError>;
}
