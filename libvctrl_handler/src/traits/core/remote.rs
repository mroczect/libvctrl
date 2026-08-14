//! Defines the `Remote` trait for interacting with remote repositories.
//!
//! # Purpose
//!
//! The `Remote` trait abstracts communication with a remote repository,
//! such as listing remote references, fetching objects, and pushing local
//! changes. This allows porcelain commands like `fetch`, `push`, and `pull`
//! to work with different protocols (HTTP, SSH, custom) without coupling
//! to a specific implementation.
//!
//! # Why a separate module
//!
//! Remote interaction is a distinct concern from local storage, indexing,
//! or configuration. Keeping the trait in its own file follows the same
//! pattern as other core traits, enabling independent backends.
//!
//! # Examples
//!
//! A dummy implementation that returns no references:
//!
//! ```
//! use libvctrl_handler::{Remote, VctrlError};
//!
//! struct DummyRemote;
//!
//! impl Remote for DummyRemote {
//!     type RefSpec = String;
//!     type RemoteRef = String;
//!
//!     fn list_refs(&self) -> Result<Vec<Self::RemoteRef>, VctrlError> {
//!         Ok(vec![])
//!     }
//!
//!     fn fetch(&mut self, _refspecs: &[Self::RefSpec]) -> Result<(), VctrlError> {
//!         Ok(())
//!     }
//!
//!     fn push(&mut self, _refspecs: &[Self::RefSpec]) -> Result<(), VctrlError> {
//!         Ok(())
//!     }
//! }
//!
//! let mut remote = DummyRemote;
//! assert!(remote.list_refs().unwrap().is_empty());
//! remote.fetch(&[]).unwrap();
//! remote.push(&[]).unwrap();
//! ```

use crate::VctrlError;

/// Trait for interacting with a remote repository.
///
/// # Purpose
///
/// `Remote` abstracts operations against a remote repository, such as
/// listing references, fetching objects, and pushing local changes. It
/// enables different transport protocols to be used interchangeably behind
/// a common interface.
///
/// # Associated Types
///
/// - `RefSpec`: the type representing a refspec (e.g., `String` or a
///   dedicated `Refspec` type).
/// - `RemoteRef`: the type representing a remote reference returned by
///   `list_refs`.
///
/// # Examples
///
/// A dummy implementation:
///
/// ```
/// use libvctrl_handler::{Remote, VctrlError};
///
/// struct DummyRemote;
///
/// impl Remote for DummyRemote {
///     type RefSpec = String;
///     type RemoteRef = String;
///
///     fn list_refs(&self) -> Result<Vec<Self::RemoteRef>, VctrlError> {
///         Ok(vec![])
///     }
///
///     fn fetch(&mut self, _refspecs: &[Self::RefSpec]) -> Result<(), VctrlError> {
///         Ok(())
///     }
///
///     fn push(&mut self, _refspecs: &[Self::RefSpec]) -> Result<(), VctrlError> {
///         Ok(())
///     }
/// }
///
/// let mut remote = DummyRemote;
/// assert!(remote.list_refs().unwrap().is_empty());
/// ```
///
/// # Errors
///
/// - [`VctrlError::IoError`] if the underlying transport fails.
/// - [`VctrlError::Other`] for protocol-level errors.
pub trait Remote {
    /// The type used to represent a refspec.
    type RefSpec;

    /// The type used to represent a remote reference.
    type RemoteRef;

    /// Lists all references available on the remote.
    ///
    /// # Errors
    ///
    /// Returns an error if the remote cannot be contacted or the reference
    /// list cannot be parsed.
    fn list_refs(&self) -> Result<Vec<Self::RemoteRef>, VctrlError>;

    /// Fetches objects and references from the remote according to the given
    /// refspecs.
    ///
    /// # Parameters
    ///
    /// - `refspecs`: a slice of refspecs describing what to fetch.
    ///
    /// # Errors
    ///
    /// Returns an error if the fetch operation fails.
    fn fetch(&mut self, refspecs: &[Self::RefSpec]) -> Result<(), VctrlError>;

    /// Pushes local objects and references to the remote according to the
    /// given refspecs.
    ///
    /// # Parameters
    ///
    /// - `refspecs`: a slice of refspecs describing what to push.
    ///
    /// # Errors
    ///
    /// Returns an error if the push operation fails.
    fn push(&mut self, refspecs: &[Self::RefSpec]) -> Result<(), VctrlError>;
}
