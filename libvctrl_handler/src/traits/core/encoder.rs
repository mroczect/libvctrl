//! Object encoder trait.

use crate::errors::VctrlError;
use crate::types::{Blob, Commit, Tag, Tree};
/// Trait for encoding structured Git objects into raw bytes.
pub trait Encoder {
    /// Encodes a blob object into raw bytes.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if encoding fails.
    fn encode_blob(&self, blob: &Blob) -> Result<Vec<u8>, VctrlError>;

    /// Encodes a tree object into raw bytes.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if encoding fails.
    fn encode_tree(&self, tree: &Tree) -> Result<Vec<u8>, VctrlError>;

    /// Encodes a commit object into raw bytes.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if encoding fails.
    fn encode_commit(&self, commit: &Commit) -> Result<Vec<u8>, VctrlError>;

    /// Encodes a tag object into raw bytes.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if encoding fails.
    fn encode_tag(&self, tag: &Tag) -> Result<Vec<u8>, VctrlError>;
}
