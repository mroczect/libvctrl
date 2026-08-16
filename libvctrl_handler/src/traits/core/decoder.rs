use crate::errors::VctrlError;
use crate::types::{Blob, Commit, Tag, Tree};
use std::io::Read;

/// Trait for decoding raw Git object bytes into structured types.
pub trait Decoder: Send + Sync {
    /// Decodes a blob object from a reader.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if decoding fails.
    fn decode_blob<R: Read + Send>(&self, reader: R) -> Result<Blob, VctrlError>;

    /// Decodes a tree object from a reader.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if decoding fails.
    fn decode_tree<R: Read + Send>(&self, reader: R) -> Result<Tree, VctrlError>;

    /// Decodes a commit object from a reader.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if decoding fails.
    fn decode_commit<R: Read + Send>(&self, reader: R) -> Result<Commit, VctrlError>;

    /// Decodes a tag object from a reader.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if decoding fails.
    fn decode_tag<R: Read + Send>(&self, reader: R) -> Result<Tag, VctrlError>;
}
