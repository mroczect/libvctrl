//! Fetching and pushing objects to and from remote backends.
//!
//! # Purpose
//!
//! This module defines the [`Transport`] trait, which abstracts the
//! communication layer required to synchronize version control objects
//! between a local repository and a remote endpoint. A transport is
//! responsible for two fundamental operations:
//!
//! - Fetching an object identified by its [`Hash`] from a remote.
//! - Pushing a locally available object to a remote.
//!
//! The trait intentionally focuses only on object movement. It does not
//! define protocols, authentication, or discovery mechanisms; those concerns
//! belong to concrete implementations.
//!
//! # Design Rationale
//!
//! The transport layer is separated from the local object store
//! ([`ObjectStore`](crate::ObjectStore)) for several reasons:
//!
//! - **Different lifecycles**: A local store is typically long-lived and
//!   disk-backed, while a transport represents a short-lived network session.
//! - **Different failure modes**: Transports may fail due to network
//!   interruption, authentication errors, or remote rejection, which are
//!   distinct from local storage failures.
//! - **Testability**: Dummy or in-memory transports make it easy to test
//!   synchronization logic without real network access.
//! - **Backend flexibility**: A transport can be implemented over HTTP,
//!   SSH, custom protocols, or even an in-process channel, without changing
//!   the core synchronization code.
//!
//! # Method Signature Rationale
//!
//! - [`fetch_object`](Transport::fetch_object) takes `&Hash` rather than an
//!   owned [`Hash`] to avoid copying the 64-byte key on the stack. It
//!   returns the object bytes as a [`Vec<u8>`] because the complete remote
//!   object is needed locally.
//! - [`push_object`](Transport::push_object) takes `&Hash` and `&[u8]` to
//!   avoid unnecessary ownership transfer. The hash identifies the object on
//!   the remote, while the byte slice carries the raw serialized content.
//!
//! # Error Handling
//!
//! Both methods return [`Result<_, VctrlError>`] to provide a unified error
//! surface. Common error variants include:
//!
//! - [`VctrlError::ObjectNotFound`](crate::VctrlError::ObjectNotFound) when
//!   the remote does not have the requested object.
//! - [`VctrlError::IoError`](crate::VctrlError::IoError) for network and
//!   transport-level failures.
//! - [`VctrlError::Other`](crate::VctrlError::Other) for protocol-specific
//!   or remote-rejection errors.
//!
//! # Internal Mechanism
//!
//! A concrete transport implementation will typically maintain some form of
//! connection state (socket, HTTP client, or in-memory map). The
//! [`fetch_object`](Transport::fetch_object) method sends a request for the
//! hash and returns the received bytes. The
//! [`push_object`](Transport::push_object) method sends the hash and data to
//! the remote for storage. The exact wire format is implementation-defined.
//!
//! # Examples
//!
//! A complete in-memory transport implementation:
//!
//! ```
//! use libvctrl_handler::{Hash, Transport, VctrlError};
//! use std::collections::HashMap;
//!
//! #[derive(Default)]
//! struct InMemoryTransport(HashMap<Hash, Vec<u8>>);
//!
//! impl Transport for InMemoryTransport {
//!     fn fetch_object(&self, hash: &Hash) -> Result<Vec<u8>, VctrlError> {
//!         self.0
//!             .get(hash)
//!             .cloned()
//!             .ok_or_else(|| VctrlError::ObjectNotFound(*hash))
//!     }
//!
//!     fn push_object(&mut self, hash: &Hash, data: &[u8]) -> Result<(), VctrlError> {
//!         self.0.insert(*hash, data.to_vec());
//!         Ok(())
//!     }
//! }
//!
//! let mut transport = InMemoryTransport::default();
//! let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
//! transport.push_object(&hash, b"data").unwrap();
//! assert_eq!(transport.fetch_object(&hash).unwrap(), b"data");
//! ```

use crate::errors::VctrlError;
use crate::types::hash::Hash;

/// Defines the interface for synchronizing objects with a remote backend.
///
/// # Purpose
///
/// A `Transport` abstracts the network or inter-process communication layer
/// required to fetch and push version control objects between a local
/// [`ObjectStore`](crate::ObjectStore) and a remote endpoint. It is the
/// bridge that enables distributed version control operations such as clone,
/// fetch, push, and pull.
///
/// # Design Rationale
///
/// - **`fetch_object` takes `&Hash`**: The method borrows the hash to avoid
///   copying the 64-byte key on the stack. Since the hash is only used for
///   lookup, borrowing is sufficient and more efficient.
/// - **`push_object` takes raw bytes**: The method receives the object data
///   as `&[u8]` and the hash as `&Hash`. The hash tells the remote where to
///   store the object, and the slice carries the payload. Borrowing avoids
///   unnecessary moves.
/// - **Distinct from [`ObjectStore`]**: The local object store is optimized
///   for persistent, content-addressed storage, while a transport is a
///   communication channel. Keeping them separate allows the local store to
///   be disk-based while the transport is purely network-oriented.
///
/// # Why Not Streaming?
///
/// Unlike [`ObjectStore::get`](crate::ObjectStore::get), which returns a
/// streaming reader, [`fetch_object`](Transport::fetch_object) returns a
/// complete [`Vec<u8>`]. This choice simplifies remote protocol interactions
/// where the entire object must be received before it can be validated or
/// stored. Streaming transport protocols can still be implemented internally
/// by the concrete transport.
///
/// # Error Handling
///
/// All methods return [`Result<_, VctrlError>`] to preserve the crate's
/// unified error model. Implementations should map network and protocol
/// errors to the appropriate variants, especially
/// [`VctrlError::IoError`](crate::VctrlError::IoError) and
/// [`VctrlError::Other`](crate::VctrlError::Other).
///
/// # Internal Mechanism
///
/// A transport implementation maintains whatever state is necessary for its
/// communication channel. For example, an HTTP transport may keep an HTTP
/// client and base URL. The methods translate the high-level object requests
/// into protocol-specific operations and translate the responses back into
/// Rust data types.
///
/// # Examples
///
/// A complete in-memory transport:
///
/// ```
/// use libvctrl_handler::{Hash, Transport, VctrlError};
/// use std::collections::HashMap;
///
/// #[derive(Default)]
/// struct InMemoryTransport(HashMap<Hash, Vec<u8>>);
///
/// impl Transport for InMemoryTransport {
///     fn fetch_object(&self, hash: &Hash) -> Result<Vec<u8>, VctrlError> {
///         self.0
///             .get(hash)
///             .cloned()
///             .ok_or_else(|| VctrlError::ObjectNotFound(*hash))
///     }
///
///     fn push_object(&mut self, hash: &Hash, data: &[u8]) -> Result<(), VctrlError> {
///         self.0.insert(*hash, data.to_vec());
///         Ok(())
///     }
/// }
///
/// let mut transport = InMemoryTransport::default();
/// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
/// transport.push_object(&hash, b"data").unwrap();
/// assert_eq!(transport.fetch_object(&hash).unwrap(), b"data");
/// ```
pub trait Transport {
    /// Fetches an object from the remote backend.
    ///
    /// # Purpose
    ///
    /// Requests the object identified by `hash` from the remote endpoint and
    /// returns its raw serialized bytes. This is the primary read operation
    /// for synchronization.
    ///
    /// # Arguments
    ///
    /// * `hash` - The content address of the object to fetch. It is borrowed
    ///   to avoid copying the 64-byte value.
    ///
    /// # Returns
    ///
    /// Returns `Ok(bytes)` containing the raw object data if the remote has
    /// the object.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::ObjectNotFound`] if the remote does not have
    /// the requested object. Returns [`VctrlError::IoError`] on network
    /// failures such as timeouts, connection resets, or DNS errors.
    ///
    /// # How It Works Internally
    ///
    /// The implementation sends a request for the hash over its
    /// communication channel. If the remote responds with the object data,
    /// the bytes are returned to the caller. If the remote reports that the
    /// object is missing, an [`ObjectNotFound`](VctrlError::ObjectNotFound)
    /// error is returned.
    ///
    /// # Examples
    ///
    /// Fetching an object from an in-memory transport:
    ///
    /// ```
    /// # use libvctrl_handler::{Hash, Transport, VctrlError};
    /// # use std::collections::HashMap;
    /// # #[derive(Default)]
    /// # struct TransportImpl(HashMap<Hash, Vec<u8>>);
    /// # impl Transport for TransportImpl {
    /// #     fn fetch_object(&self, h: &Hash) -> Result<Vec<u8>, VctrlError> {
    /// #         self.0.get(h).cloned().ok_or_else(|| VctrlError::ObjectNotFound(*h))
    /// #     }
    /// #     fn push_object(&mut self, h: &Hash, d: &[u8]) -> Result<(), VctrlError> {
    /// #         self.0.insert(*h, d.to_vec()); Ok(())
    /// #     }
    /// # }
    /// let mut transport = TransportImpl::default();
    /// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// transport.push_object(&hash, b"remote").unwrap();
    /// assert_eq!(transport.fetch_object(&hash).unwrap(), b"remote");
    /// ```
    fn fetch_object(&self, hash: &Hash) -> Result<Vec<u8>, VctrlError>;

    /// Pushes an object to the remote backend.
    ///
    /// # Purpose
    ///
    /// Uploads a raw object identified by `hash` to the remote endpoint. This
    /// is the primary write operation for synchronization, allowing local
    /// objects to be shared with other repositories.
    ///
    /// # Arguments
    ///
    /// * `hash` - The content address of the object being pushed. It is
    ///   borrowed to avoid copying the 64-byte value.
    /// * `data` - The raw serialized bytes of the object to be stored
    ///   remotely.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::IoError`] on network failures or if the remote
    /// rejects the object. A rejection may occur if the remote considers the
    /// object invalid or if the push is not permitted by access control.
    ///
    /// # How It Works Internally
    ///
    /// The implementation sends both the hash and the raw object bytes to
    /// the remote endpoint using its communication protocol. The remote may
    /// verify the hash against the content, store the object, or reject the
    /// push. If the operation is successful, `Ok(())` is returned.
    ///
    /// # Examples
    ///
    /// Pushing an object to an in-memory transport:
    ///
    /// ```
    /// # use libvctrl_handler::{Hash, Transport, VctrlError};
    /// # use std::collections::HashMap;
    /// # #[derive(Default)]
    /// # struct TransportImpl(HashMap<Hash, Vec<u8>>);
    /// # impl Transport for TransportImpl {
    /// #     fn fetch_object(&self, h: &Hash) -> Result<Vec<u8>, VctrlError> {
    /// #         self.0.get(h).cloned().ok_or_else(|| VctrlError::ObjectNotFound(*h))
    /// #     }
    /// #     fn push_object(&mut self, h: &Hash, d: &[u8]) -> Result<(), VctrlError> {
    /// #         self.0.insert(*h, d.to_vec()); Ok(())
    /// #     }
    /// # }
    /// let mut transport = TransportImpl::default();
    /// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// transport.push_object(&hash, b"payload").unwrap();
    /// assert!(transport.fetch_object(&hash).is_ok());
    /// ```
    fn push_object(&mut self, hash: &Hash, data: &[u8]) -> Result<(), VctrlError>;
}
