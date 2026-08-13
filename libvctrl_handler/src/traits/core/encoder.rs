//! Serialization of version control objects into byte vectors.
//!
//! # Purpose
//!
//! This module defines the [`Encoder`] trait, which converts high-level
//! version control objects ([`Blob`](crate::Blob), [`Tree`](crate::Tree),
//! [`Commit`](crate::Commit), [`Tag`](crate::Tag)) into byte vectors
//! suitable for storage in an [`ObjectStore`](crate::ObjectStore) or
//! transmission via a [`Transport`](crate::Transport). It is the inverse of
//! [`Decoder`](crate::Decoder).
//!
//! # Design Rationale
//!
//! The trait provides separate methods for each object type rather than a
//! generic `encode<T>(&self, obj: &T)` for several reasons:
//!
//! - It avoids requiring all object types to implement a common trait.
//! - It allows encoder implementations to handle type-specific formatting.
//! - It keeps the domain data structures pure and decoupled from the
//!   serialization interface.
//!
//! Encoding is fallible because an encoder may encounter unsupported
//! features, invalid internal state, or I/O errors during the process.
//! Therefore every method returns [`Result<Vec<u8>, VctrlError>`](crate::VctrlError).
//!
//! # Internal Mechanism
//!
//! A typical encoder implementation will access the fields of an object via
//! its public accessor methods, format them according to the chosen wire
//! format, and append them to a byte vector. The exact format is
//! implementation-defined; the trait only defines the contract.
//!
//! # Examples
//!
//! A complete dummy encoder implementation:
//!
//! ```
//! use libvctrl_handler::{Blob, Commit, Encoder, Hash, Tag, Tree, UserID, VctrlError};
//!
//! struct DummyEncoder;
//!
//! impl Encoder for DummyEncoder {
//!     fn encode_blob(&self, blob: &Blob) -> Result<Vec<u8>, VctrlError> {
//!         Ok(blob.data().to_vec())
//!     }
//!
//!     fn encode_tree(&self, _tree: &Tree) -> Result<Vec<u8>, VctrlError> {
//!         Ok(vec![])
//!     }
//!
//!     fn encode_commit(&self, _commit: &Commit) -> Result<Vec<u8>, VctrlError> {
//!         Ok(vec![])
//!     }
//!
//!     fn encode_tag(&self, _tag: &Tag) -> Result<Vec<u8>, VctrlError> {
//!         Ok(vec![])
//!     }
//! }
//!
//! let encoder = DummyEncoder;
//! let blob = Blob::new(b"data".to_vec());
//! assert_eq!(encoder.encode_blob(&blob).unwrap(), b"data");
//! ```

use crate::errors::VctrlError;
use crate::types::blob::Blob;
use crate::types::commit::Commit;
use crate::types::tag::Tag;
use crate::types::tree::Tree;

/// Defines the interface for serializing version control objects.
///
/// # Purpose
///
/// An `Encoder` translates in-memory data structures like [`Blob`] and
/// [`Commit`] into byte vectors suitable for storage in an
/// [`ObjectStore`](crate::ObjectStore) or transmission via a
/// [`Transport`](crate::Transport).
///
/// # Design Rationale
///
/// The trait provides separate methods for each object type rather than a
/// generic `encode<T>(&self, obj: &T)` to avoid requiring objects to
/// implement a shared trait, keeping the data structs pure and decoupled.
/// This design also permits specialized formatting for each object type.
///
/// # Why `&self`?
///
/// The methods take `&self` to allow a single encoder instance to be reused
/// for multiple encoding operations. Implementations may hold internal
/// buffers or configuration, and borrowing prevents unnecessary cloning of
/// the encoder itself.
///
/// # How It Works Internally
///
/// An implementation retrieves the necessary fields from the object via
/// accessor methods (e.g., [`Blob::data`](crate::Blob::data),
/// [`Commit::tree`](crate::Commit::tree)), formats them according to the
/// chosen serialization format, and writes the resulting bytes into a
/// [`Vec<u8>`]. The exact binary layout is not specified by this trait.
///
/// # Examples
///
/// A complete dummy encoder implementation:
///
/// ```
/// use libvctrl_handler::{Blob, Commit, Encoder, Hash, Tag, Tree, UserID, VctrlError};
///
/// struct DummyEncoder;
///
/// impl Encoder for DummyEncoder {
///     fn encode_blob(&self, blob: &Blob) -> Result<Vec<u8>, VctrlError> {
///         Ok(blob.data().to_vec())
///     }
///
///     fn encode_tree(&self, _tree: &Tree) -> Result<Vec<u8>, VctrlError> {
///         Ok(vec![])
///     }
///
///     fn encode_commit(&self, _commit: &Commit) -> Result<Vec<u8>, VctrlError> {
///         Ok(vec![])
///     }
///
///     fn encode_tag(&self, _tag: &Tag) -> Result<Vec<u8>, VctrlError> {
///         Ok(vec![])
///     }
/// }
///
/// let encoder = DummyEncoder;
/// let blob = Blob::new(b"data".to_vec());
/// assert_eq!(encoder.encode_blob(&blob).unwrap(), b"data");
/// ```
pub trait Encoder {
    /// Encodes a [`Blob`](crate::Blob) into its serialized byte representation.
    ///
    /// # Purpose
    ///
    /// Converts a [`Blob`] into a byte vector. The simplest implementation
    /// simply copies the blob's data; more complex formats may include
    /// headers or length prefixes.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::SerializationError`](crate::VctrlError::SerializationError)
    /// if the encoder fails to serialize the blob.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::{Blob, Commit, Encoder, Tag, Tree, VctrlError};
    /// # struct EncoderImpl;
    /// # impl Encoder for EncoderImpl {
    /// #     fn encode_blob(&self, b: &Blob) -> Result<Vec<u8>, VctrlError> { Ok(b.data().to_vec()) }
    /// #     fn encode_tree(&self, _t: &Tree) -> Result<Vec<u8>, VctrlError> { Ok(vec![]) }
    /// #     fn encode_commit(&self, _c: &Commit) -> Result<Vec<u8>, VctrlError> { Ok(vec![]) }
    /// #     fn encode_tag(&self, _t: &Tag) -> Result<Vec<u8>, VctrlError> { Ok(vec![]) }
    /// # }
    /// let encoder = EncoderImpl;
    /// let blob = Blob::new(b"data".to_vec());
    /// assert_eq!(encoder.encode_blob(&blob).unwrap(), b"data");
    /// ```
    fn encode_blob(&self, blob: &Blob) -> Result<Vec<u8>, VctrlError>;

    /// Encodes a [`Tree`](crate::Tree) into its serialized byte representation.
    ///
    /// # Purpose
    ///
    /// Converts a [`Tree`] into a byte vector. The implementation must walk
    /// the tree's entries and serialize each one according to the wire
    /// format.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::SerializationError`](crate::VctrlError::SerializationError)
    /// if the encoder fails to serialize the tree.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::{Blob, Commit, Encoder, EntryKind, Hash, Tag, Tree, TreeEntry, VctrlError};
    /// # struct EncoderImpl;
    /// # impl Encoder for EncoderImpl {
    /// #     fn encode_blob(&self, _b: &Blob) -> Result<Vec<u8>, VctrlError> { Ok(vec![]) }
    /// #     fn encode_tree(&self, t: &Tree) -> Result<Vec<u8>, VctrlError> { Ok(format!("{:?}", t.entries()).into_bytes()) }
    /// #     fn encode_commit(&self, _c: &Commit) -> Result<Vec<u8>, VctrlError> { Ok(vec![]) }
    /// #     fn encode_tag(&self, _t: &Tag) -> Result<Vec<u8>, VctrlError> { Ok(vec![]) }
    /// # }
    /// let encoder = EncoderImpl;
    /// let tree = Tree::new(vec![]).unwrap();
    /// assert!(encoder.encode_tree(&tree).is_ok());
    /// ```
    fn encode_tree(&self, tree: &Tree) -> Result<Vec<u8>, VctrlError>;

    /// Encodes a [`Commit`](crate::Commit) into its serialized byte representation.
    ///
    /// # Purpose
    ///
    /// Converts a [`Commit`] into a byte vector. The implementation must
    /// serialize the root tree hash, parent hashes, author and committer
    /// information, message, and optional metadata.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::SerializationError`](crate::VctrlError::SerializationError)
    /// if the encoder fails to serialize the commit.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::{Blob, Commit, Encoder, Hash, Tag, Tree, UserID, VctrlError};
    /// # struct EncoderImpl;
    /// # impl Encoder for EncoderImpl {
    /// #     fn encode_blob(&self, _b: &Blob) -> Result<Vec<u8>, VctrlError> { Ok(vec![]) }
    /// #     fn encode_tree(&self, _t: &Tree) -> Result<Vec<u8>, VctrlError> { Ok(vec![]) }
    /// #     fn encode_commit(&self, _c: &Commit) -> Result<Vec<u8>, VctrlError> { Ok(vec![]) }
    /// #     fn encode_tag(&self, _t: &Tag) -> Result<Vec<u8>, VctrlError> { Ok(vec![]) }
    /// # }
    /// let encoder = EncoderImpl;
    /// let tree = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// let user = UserID::new("a".to_string(), "b".to_string()).unwrap();
    /// let commit = Commit::new(tree, vec![], user.clone(), user, "msg".to_string());
    /// assert!(encoder.encode_commit(&commit).is_ok());
    /// ```
    fn encode_commit(&self, commit: &Commit) -> Result<Vec<u8>, VctrlError>;

    /// Encodes a [`Tag`](crate::Tag) into its serialized byte representation.
    ///
    /// # Purpose
    ///
    /// Converts a [`Tag`] into a byte vector. The implementation must
    /// serialize the tag name, target hash, optional tagger information,
    /// message, and metadata.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::SerializationError`](crate::VctrlError::SerializationError)
    /// if the encoder fails to serialize the tag.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::{Blob, Commit, Encoder, Hash, Tag, Tree, VctrlError};
    /// # struct EncoderImpl;
    /// # impl Encoder for EncoderImpl {
    /// #     fn encode_blob(&self, _b: &Blob) -> Result<Vec<u8>, VctrlError> { Ok(vec![]) }
    /// #     fn encode_tree(&self, _t: &Tree) -> Result<Vec<u8>, VctrlError> { Ok(vec![]) }
    /// #     fn encode_commit(&self, _c: &Commit) -> Result<Vec<u8>, VctrlError> { Ok(vec![]) }
    /// #     fn encode_tag(&self, _t: &Tag) -> Result<Vec<u8>, VctrlError> { Ok(vec![]) }
    /// # }
    /// let encoder = EncoderImpl;
    /// let target = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// let tag = Tag::new("v1".to_string(), target, None, "msg".to_string()).unwrap();
    /// assert!(encoder.encode_tag(&tag).is_ok());
    /// ```
    fn encode_tag(&self, tag: &Tag) -> Result<Vec<u8>, VctrlError>;
}
