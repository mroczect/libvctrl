//! Serialization of version control objects into byte vectors.

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
/// [`Commit`] into byte vectors suitable for storage in an [`ObjectStore`]
/// or transmission via a [`Transport`].
///
/// # Design Rationale
///
/// The trait provides separate methods for each object type rather than a
/// generic `encode<T>(&self, obj: &T)` to avoid requiring objects to implement
/// a shared trait, keeping the data structs pure and decoupled.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::{Blob, Commit, Encoder, Tag, Tree, VctrlError};
///
/// struct DummyEncoder;
/// impl Encoder for DummyEncoder {
///     fn encode_blob(&self, blob: &Blob) -> Result<Vec<u8>, VctrlError> {
///         Ok(blob.data().to_vec())
///     }
///     fn encode_tree(&self, _tree: &Tree) -> Result<Vec<u8>, VctrlError> { Ok(vec![]) }
///     fn encode_commit(&self, _commit: &Commit) -> Result<Vec<u8>, VctrlError> { Ok(vec![]) }
///     fn encode_tag(&self, _tag: &Tag) -> Result<Vec<u8>, VctrlError> { Ok(vec![]) }
/// }
///
/// let encoder = DummyEncoder;
/// let blob = Blob::new(b"data".to_vec());
/// assert_eq!(encoder.encode_blob(&blob).unwrap(), b"data");
/// ```
pub trait Encoder {
    /// Encodes a [`Blob`] into its serialized byte representation.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::SerializationError`] if the encoder fails to
    /// serialize the blob.
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

    /// Encodes a [`Tree`] into its serialized byte representation.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::SerializationError`] if the encoder fails to
    /// serialize the tree.
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

    /// Encodes a [`Commit`] into its serialized byte representation.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::SerializationError`] if the encoder fails to
    /// serialize the commit.
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

    /// Encodes a [`Tag`] into its serialized byte representation.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::SerializationError`] if the encoder fails to
    /// serialize the tag.
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
