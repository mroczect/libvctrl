//! Deserialization of version control objects from byte slices.
//!
//! # Purpose
//!
//! This module defines the [`Decoder`] trait, which is the inverse of
//! [`Encoder`](crate::Encoder). A decoder translates raw byte vectors back
//! into high-level version control objects such as [`Blob`](crate::Blob),
//! [`Tree`](crate::Tree), [`Commit`](crate::Commit), and [`Tag`](crate::Tag).
//! The trait is intentionally abstract, allowing multiple serialization
//! formats without coupling to any specific representation.
//!
//! # Design Rationale
//!
//! Decoding is a fallible operation because byte slices may be corrupted,
//! truncated, or malformed. Every method therefore returns
//! [`Result<_, VctrlError>`](crate::VctrlError), with
//! [`VctrlError::CorruptedData`](crate::VctrlError::CorruptedData) as the
//! primary error variant for invalid input. This forces callers to handle
//! failure explicitly and prevents invalid objects from entering the system.
//!
//! The trait defines separate methods for each object type instead of a
//! generic `decode<T>(&self, data: &[u8]) -> Result<T, VctrlError>` because:
//!
//! - It avoids requiring all object types to implement a common trait.
//! - It allows decoders to perform type-specific validation and parsing.
//! - It keeps the data structures pure and decoupled from the decoding
//!   interface.
//!
//! # Internal Mechanism
//!
//! A typical decoder implementation will parse the byte slice according to a
//! predefined wire format, validate structural invariants (e.g., hash
//! lengths, name lengths, sort order), and then call the appropriate
//! constructor for the object type. The constructors themselves perform
//! additional validation, so a decoder can often delegate to them and
//! propagate errors directly.
//!
//! # Examples
//!
//! A complete dummy decoder implementation:
//!
//! ```
//! use libvctrl_handler::{Blob, Commit, Decoder, Hash, Tag, Tree, UserID, VctrlError};
//!
//! struct DummyDecoder;
//!
//! impl Decoder for DummyDecoder {
//!     fn decode_blob(&self, data: &[u8]) -> Result<Blob, VctrlError> {
//!         Ok(Blob::new(data.to_vec()))
//!     }
//!
//!     fn decode_tree(&self, _data: &[u8]) -> Result<Tree, VctrlError> {
//!         Tree::new(vec![])
//!     }
//!
//!     fn decode_commit(&self, _data: &[u8]) -> Result<Commit, VctrlError> {
//!         let tree = Hash::from_bytes(&[0u8; 64])?;
//!         let user = UserID::new("a".to_string(), "b".to_string())?;
//!         Ok(Commit::new(tree, vec![], user.clone(), user, String::new()))
//!     }
//!
//!     fn decode_tag(&self, _data: &[u8]) -> Result<Tag, VctrlError> {
//!         let target = Hash::from_bytes(&[0u8; 64])?;
//!         Tag::new("tag".to_string(), target, None, String::new())
//!     }
//! }
//!
//! let decoder = DummyDecoder;
//! let blob = decoder.decode_blob(b"data").unwrap();
//! assert_eq!(blob.data(), b"data");
//! ```

use crate::errors::VctrlError;
use crate::types::blob::Blob;
use crate::types::commit::Commit;
use crate::types::tag::Tag;
use crate::types::tree::Tree;

/// Defines the interface for deserializing version control objects.
///
/// # Purpose
///
/// A `Decoder` translates byte vectors back into in-memory data structures.
/// It is the inverse of [`Encoder`](crate::Encoder). The trait is
/// object-specialized: each method decodes exactly one object type, allowing
/// implementations to handle type-specific parsing and validation.
///
/// # Design Rationale
///
/// Decoding can fail due to corrupted data, malformed inputs, or version
/// mismatches, hence every method returns a [`Result`] with
/// [`VctrlError`]. By keeping the trait methods separate, we avoid the need
/// for objects to share a common interface and preserve the purity of the
/// domain types.
///
/// # Why `&self`?
///
/// The methods take `&self` rather than consuming the decoder. This allows a
/// single decoder instance to be reused for multiple decode operations,
/// which is important for streaming or stateful decoders.
///
/// # How It Works Internally
///
/// An implementation reads the byte slice and reconstructs the object.
/// Validation is typically delegated to the object constructors (e.g.,
/// [`Tree::new`](crate::Tree::new), [`Tag::new`](crate::Tag::new)), which
/// enforce invariants such as name validity and sort order. If any
/// validation fails, the decoder returns
/// [`VctrlError::CorruptedData`](crate::VctrlError::CorruptedData) or a more
/// specific variant depending on the context.
///
/// # Examples
///
/// A complete dummy decoder implementation:
///
/// ```
/// use libvctrl_handler::{Blob, Commit, Decoder, Hash, Tag, Tree, UserID, VctrlError};
///
/// struct DummyDecoder;
///
/// impl Decoder for DummyDecoder {
///     fn decode_blob(&self, data: &[u8]) -> Result<Blob, VctrlError> {
///         Ok(Blob::new(data.to_vec()))
///     }
///
///     fn decode_tree(&self, _data: &[u8]) -> Result<Tree, VctrlError> {
///         Tree::new(vec![])
///     }
///
///     fn decode_commit(&self, _data: &[u8]) -> Result<Commit, VctrlError> {
///         let tree = Hash::from_bytes(&[0u8; 64])?;
///         let user = UserID::new("a".to_string(), "b".to_string())?;
///         Ok(Commit::new(tree, vec![], user.clone(), user, String::new()))
///     }
///
///     fn decode_tag(&self, _data: &[u8]) -> Result<Tag, VctrlError> {
///         let target = Hash::from_bytes(&[0u8; 64])?;
///         Tag::new("tag".to_string(), target, None, String::new())
///     }
/// }
///
/// let decoder = DummyDecoder;
/// let blob = decoder.decode_blob(b"data").unwrap();
/// assert_eq!(blob.data(), b"data");
/// ```
pub trait Decoder {
    /// Decodes a byte slice into a [`Blob`](crate::Blob).
    ///
    /// # Purpose
    ///
    /// Reconstructs a [`Blob`] from its serialized form. The implementation
    /// should extract the raw byte content and wrap it in a new
    /// [`Blob`](crate::Blob) using [`Blob::new`](crate::Blob::new). No
    /// additional validation is required because a [`Blob`] accepts any
    /// byte sequence.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::CorruptedData`](crate::VctrlError::CorruptedData)
    /// if the byte slice does not represent a valid blob according to the
    /// wire format.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::{Blob, Commit, Decoder, Hash, Tag, Tree, UserID, VctrlError};
    ///
    /// struct DecoderImpl;
    ///
    /// impl Decoder for DecoderImpl {
    ///     fn decode_blob(&self, data: &[u8]) -> Result<Blob, VctrlError> {
    ///         Ok(Blob::new(data.to_vec()))
    ///     }
    /// #     fn decode_tree(&self, _data: &[u8]) -> Result<Tree, VctrlError> {
    /// #         Tree::new(vec![])
    /// #     }
    /// #     fn decode_commit(&self, _data: &[u8]) -> Result<Commit, VctrlError> {
    /// #         let tree = Hash::from_bytes(&[0u8; 64])?;
    /// #         let user = UserID::new("a".to_string(), "b".to_string())?;
    /// #         Ok(Commit::new(tree, vec![], user.clone(), user, String::new()))
    /// #     }
    /// #     fn decode_tag(&self, _data: &[u8]) -> Result<Tag, VctrlError> {
    /// #         let target = Hash::from_bytes(&[0u8; 64])?;
    /// #         Tag::new("tag".to_string(), target, None, String::new())
    /// #     }
    /// }
    ///
    /// let decoder = DecoderImpl;
    /// let blob = decoder.decode_blob(b"data").unwrap();
    /// assert_eq!(blob.data(), b"data");
    /// ```
    fn decode_blob(&self, data: &[u8]) -> Result<Blob, VctrlError>;

    /// Decodes a byte slice into a [`Tree`](crate::Tree).
    ///
    /// # Purpose
    ///
    /// Reconstructs a [`Tree`] from its serialized representation. The
    /// implementation must parse entries, validate their names and order,
    /// and call [`Tree::new`](crate::Tree::new) with the resulting vector.
    ///
    /// # Why validation is critical
    ///
    /// Trees enforce a strict lexicographic order on entries and forbid
    /// duplicate names. Decoders must ensure that the byte slice respects
    /// these constraints; otherwise, the resulting tree would violate
    /// invariants required for deterministic hashing.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::CorruptedData`](crate::VctrlError::CorruptedData)
    /// if the byte slice contains malformed entries or violates tree
    /// invariants.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::{Blob, Commit, Decoder, Hash, Tag, Tree, UserID, VctrlError};
    /// # struct DecoderImpl;
    /// # impl Decoder for DecoderImpl {
    /// #     fn decode_blob(&self, d: &[u8]) -> Result<Blob, VctrlError> { Ok(Blob::new(d.to_vec())) }
    /// #     fn decode_tree(&self, _d: &[u8]) -> Result<Tree, VctrlError> { Tree::new(vec![]) }
    /// #     fn decode_commit(&self, _d: &[u8]) -> Result<Commit, VctrlError> { let t = Hash::from_bytes(&[0u8; 64])?; let u = UserID::new("a".to_string(), "b".to_string())?; Ok(Commit::new(t, vec![], u.clone(), u, String::new())) }
    /// #     fn decode_tag(&self, _d: &[u8]) -> Result<Tag, VctrlError> { let t = Hash::from_bytes(&[0u8; 64])?; Tag::new("t".to_string(), t, None, String::new()) }
    /// # }
    /// let decoder = DecoderImpl;
    /// let tree = decoder.decode_tree(b"").unwrap();
    /// assert!(tree.entries().is_empty());
    /// ```
    fn decode_tree(&self, data: &[u8]) -> Result<Tree, VctrlError>;

    /// Decodes a byte slice into a [`Commit`](crate::Commit).
    ///
    /// # Purpose
    ///
    /// Reconstructs a [`Commit`] from its serialized form. The implementation
    /// must extract the root tree hash, parent hashes, author and committer
    /// information, message, and optional metadata, then construct a
    /// [`Commit`](crate::Commit) using either
    /// [`Commit::new`](crate::Commit::new) or
    /// [`Commit::with_meta`](crate::Commit::with_meta).
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::CorruptedData`](crate::VctrlError::CorruptedData)
    /// if any field is malformed or fails validation.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::{Blob, Commit, Decoder, Hash, Tag, Tree, UserID, VctrlError};
    /// # struct DecoderImpl;
    /// # impl Decoder for DecoderImpl {
    /// #     fn decode_blob(&self, _d: &[u8]) -> Result<Blob, VctrlError> { Ok(Blob::new(vec![])) }
    /// #     fn decode_tree(&self, _d: &[u8]) -> Result<Tree, VctrlError> { Tree::new(vec![]) }
    /// #     fn decode_commit(&self, _d: &[u8]) -> Result<Commit, VctrlError> { let t = Hash::from_bytes(&[0u8; 64])?; let u = UserID::new("a".to_string(), "b".to_string())?; Ok(Commit::new(t, vec![], u.clone(), u, String::new())) }
    /// #     fn decode_tag(&self, _d: &[u8]) -> Result<Tag, VctrlError> { let t = Hash::from_bytes(&[0u8; 64])?; Tag::new("t".to_string(), t, None, String::new()) }
    /// # }
    /// let decoder = DecoderImpl;
    /// let commit = decoder.decode_commit(b"").unwrap();
    /// assert_eq!(commit.message(), "");
    /// ```
    fn decode_commit(&self, data: &[u8]) -> Result<Commit, VctrlError>;

    /// Decodes a byte slice into a [`Tag`](crate::Tag).
    ///
    /// # Purpose
    ///
    /// Reconstructs a [`Tag`] from its serialized representation. The
    /// implementation must extract the tag name, target hash, optional tagger
    /// information, message, and metadata, then construct a
    /// [`Tag`](crate::Tag) using either [`Tag::new`](crate::Tag::new) or
    /// [`Tag::with_meta`](crate::Tag::with_meta).
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::CorruptedData`](crate::VctrlError::CorruptedData)
    /// if the name is invalid or the target hash length is incorrect.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::{Blob, Commit, Decoder, Hash, Tag, Tree, UserID, VctrlError};
    /// # struct DecoderImpl;
    /// # impl Decoder for DecoderImpl {
    /// #     fn decode_blob(&self, _d: &[u8]) -> Result<Blob, VctrlError> { Ok(Blob::new(vec![])) }
    /// #     fn decode_tree(&self, _d: &[u8]) -> Result<Tree, VctrlError> { Tree::new(vec![]) }
    /// #     fn decode_commit(&self, _d: &[u8]) -> Result<Commit, VctrlError> { let t = Hash::from_bytes(&[0u8; 64])?; let u = UserID::new("a".to_string(), "b".to_string())?; Ok(Commit::new(t, vec![], u.clone(), u, String::new())) }
    /// #     fn decode_tag(&self, _d: &[u8]) -> Result<Tag, VctrlError> { let t = Hash::from_bytes(&[0u8; 64])?; Tag::new("t".to_string(), t, None, String::new()) }
    /// # }
    /// let decoder = DecoderImpl;
    /// let tag = decoder.decode_tag(b"").unwrap();
    /// assert_eq!(tag.name(), "t");
    /// ```
    fn decode_tag(&self, data: &[u8]) -> Result<Tag, VctrlError>;
}
