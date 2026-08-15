//! Object decoder trait.

use crate::errors::VctrlError;
use crate::types::{Blob, Commit, Tag, Tree};

/// Trait for decoding raw Git object bytes into structured types.
pub trait Decoder {
    /// Decodes a blob object from raw bytes.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if decoding fails.
    fn decode_blob(&self, data: &[u8]) -> Result<Blob, VctrlError>;

    /// Decodes a tree object from raw bytes.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if decoding fails.
    fn decode_tree(&self, data: &[u8]) -> Result<Tree, VctrlError>;

    /// Decodes a commit object from raw bytes.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if decoding fails.
    fn decode_commit(&self, data: &[u8]) -> Result<Commit, VctrlError>;

    /// Decodes a tag object from raw bytes.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if decoding fails.
    fn decode_tag(&self, data: &[u8]) -> Result<Tag, VctrlError>;
}
