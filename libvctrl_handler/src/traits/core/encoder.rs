use crate::errors::VctrlError;
use crate::types::{Blob, Commit, Tag, Tree};
use std::io::Write;

/// Trait for encoding structured Git objects into raw bytes.
pub trait Encoder: Send + Sync {
    /// Encodes a blob object into a writer.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if encoding fails.
    fn encode_blob<W: Write + Send>(&self, blob: &Blob, writer: &mut W) -> Result<(), VctrlError>;

    /// Encodes a tree object into a writer.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if encoding fails.
    fn encode_tree<W: Write + Send>(&self, tree: &Tree, writer: &mut W) -> Result<(), VctrlError>;

    /// Encodes a commit object into a writer.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if encoding fails.
    fn encode_commit<W: Write + Send>(
        &self,
        commit: &Commit,
        writer: &mut W,
    ) -> Result<(), VctrlError>;

    /// Encodes a tag object into a writer.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if encoding fails.
    fn encode_tag<W: Write + Send>(&self, tag: &Tag, writer: &mut W) -> Result<(), VctrlError>;
}
