//! Transport trait.
//!
//! # Architecture
//! This module defines the low-level contract for sending and receiving raw Git
//! objects over a network. It is distinct from the [`Remote`](crate::traits::core::remote::Remote)
//! module, which handles higher-level repository semantics like refspec negotiation.
//! The `Transport` trait acts as a dumb pipe: it merely maps object hashes to byte streams.
//!
//! # Design Rationale: Streaming I/O
//! The `fetch_object` method returns a `Box<dyn Read>` rather than a `Vec<u8>`.
//! This is a critical architectural decision for network efficiency. Git objects
//! can be massive. By returning a reader, the transport backend can stream data
//! directly from the network socket to the decoder, decompressing on the fly and
//! maintaining a constant memory footprint regardless of the object's size.

use crate::errors::VctrlError;
use crate::types::Hash;
use std::io::Read;

/// Trait for transporting Git objects.
///
/// # Why this exists
/// Provides a backend-agnostic abstraction for the raw transfer of Git objects.
/// Whether the underlying protocol is HTTP, SSH, or the Git wire protocol, this
/// trait allows the core engine to fetch missing objects or push new ones without
/// being coupled to the specific networking implementation or socket management.
///
/// # How it works
/// The trait defines two operations:
/// - `fetch_object`: Downloads an object by its hash, returning a stream.
/// - `push_object`: Uploads an object's data to the remote.
///
/// # Design Rationale: Mutability Split
/// `fetch_object` takes `&self` because it is a read-only operation from the
/// perspective of the transport's state; multiple threads can safely fetch objects
/// concurrently. Conversely, `push_object` takes `&mut self` because writing to
/// a network socket is inherently stateful and often requires sequential, exclusive
/// access to prevent interleaved data corruption.
///
/// # Examples
///
/// Implementing the trait for a mock in-memory transport:
///
/// ```
/// # use libvctrl_handler::traits::core::transport::Transport;
/// # use libvctrl_handler::{Hash, VctrlError};
/// # use std::collections::HashMap;
/// # use std::io::Cursor;
/// #
/// #[derive(Default)]
/// struct MockTransport {
///     remote_store: HashMap<Hash, Vec<u8>>,
/// }
///
/// impl Transport for MockTransport {
///     fn fetch_object(&self, hash: &Hash) -> Result<Box<dyn Read + Send + '_>, VctrlError> {
///         match self.remote_store.get(hash) {
///             Some(data) => Ok(Box::new(Cursor::new(data.clone()))),
///             None => Err(VctrlError::ObjectNotFound(*hash)),
///         }
///     }
///
///     fn push_object(&mut self, hash: &Hash, data: &[u8]) -> Result<(), VctrlError> {
///         self.remote_store.insert(*hash, data.to_vec());
///         Ok(())
///     }
/// }
///
/// let mut transport = MockTransport::default();
/// let hash = Hash::from_bytes(&[0_u8; 64])?;
/// transport.push_object(&hash, b"raw object data")?;
/// assert!(transport.fetch_object(&hash).is_ok());
/// # Ok::<(), VctrlError>(())
/// ```
pub trait Transport: Send + Sync {
    /// Fetches an object by hash, returning a reader.
    ///
    /// # How it works
    /// Requests an object from the remote endpoint using its cryptographic hash.
    /// The implementor returns a boxed reader. The lifetime `'_` ties the returned
    /// reader to the lifetime of the `Transport` instance, ensuring the underlying
    /// network socket or buffer remains valid while the stream is being consumed.
    /// This prevents loading large objects into memory all at once.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::ObjectNotFound`] if the remote does not possess the object.
    /// Returns [`VctrlError`] if a network I/O error occurs during the transfer.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::transport::Transport;
    /// # use libvctrl_handler::{Hash, VctrlError};
    /// # use std::collections::HashMap;
    /// # use std::io::{Cursor, Read};
    /// # #[derive(Default)]
    /// # struct MockTransport { remote_store: HashMap<Hash, Vec<u8>> }
    /// # impl Transport for MockTransport {
    /// #     fn fetch_object(&self, h: &Hash) -> Result<Box<dyn Read + Send + '_>, VctrlError> {
    /// #         match self.remote_store.get(h) { Some(d) => Ok(Box::new(Cursor::new(d.clone()))), None => Err(VctrlError::ObjectNotFound(*h)) }
    /// #     }
    /// #     fn push_object(&mut self, h: &Hash, d: &[u8]) -> Result<(), VctrlError> {
    /// #         self.remote_store.insert(*h, d.to_vec()); Ok(())
    /// #     }
    /// # }
    /// let mut transport = MockTransport::default();
    /// let hash = Hash::from_bytes(&[1u8; 64])?;
    /// transport.push_object(&hash, b"fetch me")?;
    ///
    /// let mut reader = transport.fetch_object(&hash)?;
    /// let mut content = String::new();
    /// reader.read_to_string(&mut content)?;
    /// assert_eq!(content, "fetch me");
    /// # Ok::<(), VctrlError>(())
    /// ```
    fn fetch_object(&self, hash: &Hash) -> Result<Box<dyn Read + Send + '_>, VctrlError>;

    /// Pushes an object to the remote.
    ///
    /// # How it works
    /// Accepts the object's hash and a byte slice of its raw, uncompressed content.
    /// The implementor is responsible for transmitting this data to the remote endpoint.
    /// Requires `&mut self` to enforce exclusive access, preventing data races when
    /// multiple threads attempt to write to the same network socket simultaneously.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the network connection fails, the remote rejects
    /// the data, or an I/O error occurs during transmission.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::transport::Transport;
    /// # use libvctrl_handler::{Hash, VctrlError};
    /// # use std::collections::HashMap;
    /// # use std::io::Cursor;
    /// # #[derive(Default)]
    /// # struct MockTransport { remote_store: HashMap<Hash, Vec<u8>> }
    /// # impl Transport for MockTransport {
    /// #     fn fetch_object(&self, h: &Hash) -> Result<Box<dyn Read + Send + '_>, VctrlError> {
    /// #         match self.remote_store.get(h) { Some(d) => Ok(Box::new(Cursor::new(d.clone()))), None => Err(VctrlError::ObjectNotFound(*h)) }
    /// #     }
    /// #     fn push_object(&mut self, h: &Hash, d: &[u8]) -> Result<(), VctrlError> {
    /// #         self.remote_store.insert(*h, d.to_vec()); Ok(())
    /// #     }
    /// # }
    /// let mut transport = MockTransport::default();
    /// let hash = Hash::from_bytes(&[2u8; 64])?;
    /// transport.push_object(&hash, b"pushing data")?;
    /// # Ok::<(), VctrlError>(())
    /// ```
    fn push_object(&mut self, hash: &Hash, data: &[u8]) -> Result<(), VctrlError>;
}
