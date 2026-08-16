//! Remote repository trait.
//!
//! # Architecture
//! This module defines the abstract contract for interacting with remote repositories.
//! It abstracts the complex orchestration of network protocols (e.g., HTTP, SSH, Git)
//! into a unified interface. By using this trait, the core engine can execute fetch
//! and push operations without being coupled to the underlying transport mechanism
//! or wire protocol.
//!
//! # Design Rationale: Associated Types vs. Generics
//! The trait uses associated types (`type RefSpec`, `type RemoteRef`) rather than
//! generic parameters. This design ties the data representations directly to the
//! specific `Remote` implementation. An HTTP backend might parse refspecs into
//! structured objects, while a custom binary protocol might use raw byte slices.
//! This prevents type mismatches at compile time and simplifies the API by removing
//! the need for verbose generic annotations at every call site.

use crate::errors::VctrlError;

/// Trait for interacting with remote repositories.
///
/// # Why this exists
/// Provides a high-level interface for synchronizing state between a local
/// repository and a remote endpoint. It encapsulates the logic for discovering
/// remote references, fetching missing objects, and pushing local history.
/// Abstracting this into a trait allows the crate to support multiple remote
/// backends (e.g., standard Git, custom distributed ledgers) seamlessly.
///
/// # How it works
/// The trait defines three core operations:
/// - `list_refs`: Queries the remote for its current reference state.
/// - `fetch`: Downloads objects specified by refspecs and updates local remote-tracking branches.
/// - `push`: Uploads local objects and updates remote references.
///
/// # Design Rationale: Mutability Split
/// `list_refs` takes `&self` because it is a pure query operation that does not
/// alter the local or remote state; multiple threads can safely list refs concurrently.
/// Conversely, `fetch` and `push` take `&mut self`. These operations fundamentally
/// mutate state (updating local object stores or remote refs) and often require
/// sequential, exclusive access to network streams and internal buffers to prevent
/// data corruption or race conditions.
///
/// # Examples
///
/// Implementing the trait for a mock remote backend:
///
/// ```
/// # use libvctrl_handler::traits::core::remote::Remote;
/// # use libvctrl_handler::VctrlError;
/// #
/// #[derive(Default)]
/// struct MockRemote {
///     refs: Vec<String>,
/// }
///
/// impl Remote for MockRemote {
///     type RefSpec = String;
///     type RemoteRef = String;
///
///     fn list_refs(&self) -> Result<Vec<Self::RemoteRef>, VctrlError> {
///         Ok(self.refs.clone())
///     }
///
///     fn fetch(&mut self, _refspecs: &[Self::RefSpec]) -> Result<(), VctrlError> {
///         // Mock fetch: no-op
///         Ok(())
///     }
///
///     fn push(&mut self, _refspecs: &[Self::RefSpec]) -> Result<(), VctrlError> {
///         // Mock push: no-op
///         Ok(())
///     }
/// }
///
/// let remote = MockRemote::default();
/// assert!(remote.list_refs().is_ok());
/// # Ok::<(), VctrlError>(())
/// ```
pub trait Remote: Send + Sync {
    /// The refspec type.
    ///
    /// # Why this exists
    /// Decouples the refspec representation from the trait. A refspec defines the
    /// mapping between remote and local references (e.g., `refs/heads/*:refs/remotes/origin/*`).
    /// Allowing backends to define their own type enables protocol-specific optimizations
    /// or pre-parsed structures.
    type RefSpec: Send + Sync;

    /// The remote reference type.
    ///
    /// # Why this exists
    /// Defines the structure of a reference as advertised by the remote. This might
    /// include the hash, the name, and additional capabilities (e.g., symref targets)
    /// negotiated during the protocol handshake.
    type RemoteRef: Send + Sync;

    /// Lists references available on the remote.
    ///
    /// # How it works
    /// Connects to the remote (or queries a cached advertisement) and retrieves
    /// a list of all references (branches, tags) that the remote currently possesses.
    /// Takes `&self` as this is a read-only operation that should be safe to call
    /// concurrently.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the network connection fails, the remote is
    /// unreachable, or the protocol handshake fails.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::remote::Remote;
    /// # use libvctrl_handler::VctrlError;
    /// # #[derive(Default)]
    /// # struct MockRemote { refs: Vec<String> }
    /// # impl Remote for MockRemote {
    /// #     type RefSpec = String; type RemoteRef = String;
    /// #     fn list_refs(&self) -> Result<Vec<Self::RemoteRef>, VctrlError> { Ok(self.refs.clone()) }
    /// #     fn fetch(&mut self, _r: &[Self::RefSpec]) -> Result<(), VctrlError> { Ok(()) }
    /// #     fn push(&mut self, _r: &[Self::RefSpec]) -> Result<(), VctrlError> { Ok(()) }
    /// # }
    /// let remote = MockRemote { refs: vec!["refs/heads/main".to_string()] };
    /// let refs = remote.list_refs()?;
    /// assert_eq!(refs.len(), 1);
    /// # Ok::<(), VctrlError>(())
    /// ```
    fn list_refs(&self) -> Result<Vec<Self::RemoteRef>, VctrlError>;

    /// Fetches objects according to the given refspecs.
    ///
    /// # How it works
    /// Takes a slice of refspecs and negotiates with the remote to determine which
    /// objects are missing locally. It downloads these objects (often via a packfile),
    /// inserts them into the local object store, and updates local remote-tracking
    /// references (e.g., `refs/remotes/origin/*`). Requires `&mut self` as it
    /// modifies local state and network streams.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the network transfer fails, objects are corrupted
    /// in transit, or the local object store cannot be written to.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::remote::Remote;
    /// # use libvctrl_handler::VctrlError;
    /// # #[derive(Default)]
    /// # struct MockRemote { refs: Vec<String> }
    /// # impl Remote for MockRemote {
    /// #     type RefSpec = String; type RemoteRef = String;
    /// #     fn list_refs(&self) -> Result<Vec<Self::RemoteRef>, VctrlError> { Ok(self.refs.clone()) }
    /// #     fn fetch(&mut self, _r: &[Self::RefSpec]) -> Result<(), VctrlError> { Ok(()) }
    /// #     fn push(&mut self, _r: &[Self::RefSpec]) -> Result<(), VctrlError> { Ok(()) }
    /// # }
    /// let mut remote = MockRemote::default();
    /// let refspecs = vec!["refs/heads/main:refs/remotes/origin/main".to_string()];
    /// remote.fetch(&refspecs)?;
    /// # Ok::<(), VctrlError>(())
    /// ```
    fn fetch(&mut self, refspecs: &[Self::RefSpec]) -> Result<(), VctrlError>;

    /// Pushes objects according to the given refspecs.
    ///
    /// # How it works
    /// Takes a slice of refspecs and sends local objects to the remote that are
    /// required to satisfy the refspecs. It updates the remote references accordingly.
    /// Requires `&mut self` as it consumes network resources and may mutate internal
    /// state regarding the push process.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the remote rejects the update (e.g., non-fast-forward
    /// push), network transfer fails, or permission is denied.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::remote::Remote;
    /// # use libvctrl_handler::VctrlError;
    /// # #[derive(Default)]
    /// # struct MockRemote { refs: Vec<String> }
    /// # impl Remote for MockRemote {
    /// #     type RefSpec = String; type RemoteRef = String;
    /// #     fn list_refs(&self) -> Result<Vec<Self::RemoteRef>, VctrlError> { Ok(self.refs.clone()) }
    /// #     fn fetch(&mut self, _r: &[Self::RefSpec]) -> Result<(), VctrlError> { Ok(()) }
    /// #     fn push(&mut self, _r: &[Self::RefSpec]) -> Result<(), VctrlError> { Ok(()) }
    /// # }
    /// let mut remote = MockRemote::default();
    /// let refspecs = vec!["refs/heads/main:refs/heads/main".to_string()];
    /// remote.push(&refspecs)?;
    /// # Ok::<(), VctrlError>(())
    /// ```
    fn push(&mut self, refspecs: &[Self::RefSpec]) -> Result<(), VctrlError>;
}
