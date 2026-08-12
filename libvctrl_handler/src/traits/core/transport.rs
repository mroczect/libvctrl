//! Fetching and pushing objects to/from remote backends.

use crate::errors::VctrlError;
use crate::types::hash::Hash;

/// Defines the interface for synchronizing objects with a remote backend.
///
/// # Purpose
///
/// A `Transport` abstracts the network or inter-process communication layer
/// required to fetch and push version control objects between a local
/// [`ObjectStore`] and a remote endpoint.
///
/// # Design Rationale
///
/// `fetch_object` takes a `&Hash` to avoid copying the 64-byte key, while
/// `push_object` takes the raw bytes to be stored remotely. The trait is
/// distinct from [`ObjectStore`] to allow the local store to be disk-based
/// while the transport is purely network-oriented.
///
/// # Examples
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
///         self.0.get(hash).cloned().ok_or_else(|| VctrlError::ObjectNotFound(*hash))
///     }
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
    /// # Errors
    ///
    /// Returns [`VctrlError::ObjectNotFound`] if the remote does not have the object.
    /// Returns [`VctrlError::IoError`] on network failures.
    ///
    /// # Examples
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
    /// let mut t = TransportImpl::default();
    /// let h = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// t.push_object(&h, b"remote").unwrap();
    /// assert_eq!(t.fetch_object(&h).unwrap(), b"remote");
    /// ```
    fn fetch_object(&self, hash: &Hash) -> Result<Vec<u8>, VctrlError>;

    /// Pushes an object to the remote backend.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::IoError`] on network failures or if the remote
    /// rejects the object.
    ///
    /// # Examples
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
    /// let mut t = TransportImpl::default();
    /// let h = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// t.push_object(&h, b"payload").unwrap();
    /// assert!(t.fetch_object(&h).is_ok());
    /// ```
    fn push_object(&mut self, hash: &Hash, data: &[u8]) -> Result<(), VctrlError>;
}
