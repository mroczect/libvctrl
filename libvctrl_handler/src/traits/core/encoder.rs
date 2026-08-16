//! Object encoder trait.
//!
//! # Architecture
//! This module defines the contract for serializing strongly-typed Git domain
//! objects ([`Blob`], [`Tree`], [`Commit`], [`Tag`]) into raw byte streams.
//! It acts as the bridge between the crate's type-safe in-memory representations
//! and unstructured I/O data storage or network transmission.
//!
//! # Design Rationale: Streaming Serialization
//! Instead of returning a `Vec<u8>` or `Box<[u8]>`, the encoder methods require a
//! generic `W: Write` bound. This is a critical architectural decision: it forces
//! streaming serialization. Git objects (especially blobs) can be massive. By writing
//! directly to a stream, the encoder can process gigabytes of data with a fixed memory
//! footprint, preventing out-of-memory (OOM) errors and avoiding the CPU overhead of
//! allocating and resizing temporary heap buffers.

use crate::errors::VctrlError;
use crate::types::{Blob, Commit, Tag, Tree};
use std::io::Write;

/// Trait for encoding structured Git objects into raw bytes.
///
/// # Why this exists
/// Abstracts the serialization logic away from the storage backend. Whether objects
/// are being written to loose files on disk, compressed into a packfile, or streamed
/// over a network socket, the encoding logic remains identical. This allows the crate
/// to support multiple wire formats or compression algorithms by simply providing
/// different implementations of this trait.
///
/// # How it works
/// The trait uses generic methods (`<W: Write + Send>`) rather than dynamic trait
/// objects (`&mut dyn Write`). This design leverages Rust's monomorphization: the
/// compiler generates a specific version of the encode function for every concrete
/// writer type used at runtime. This eliminates dynamic dispatch overhead, allowing
/// the compiler to aggressively inline the writing logic and optimize away function
/// call boundaries.
///
/// # Design Rationale: Thread Safety
/// The trait requires `Send + Sync` on `Self`, and `Send` on the writer `W`. This
/// ensures that encoding operations can be safely dispatched to a thread pool. For
/// example, when writing a multi-object packfile, the engine can distribute object
/// serialization across multiple worker threads to utilize multi-core parallelism
/// without risking data races on the underlying writer or encoder state.
///
/// # Examples
///
/// Implementing the trait for a mock streaming writer:
///
/// ```
/// # use libvctrl_handler::traits::core::encoder::Encoder;
/// # use libvctrl_handler::{Blob, Commit, Tag, Tree, VctrlError};
/// # use std::io::Write;
/// #
/// struct MockEncoder;
///
/// impl Encoder for MockEncoder {
///     fn encode_blob<W: Write + Send>(&self, blob: &Blob, writer: &mut W) -> Result<(), VctrlError> {
///         // Write the raw blob data directly to the stream
///         writer.write_all(blob.data())?;
///         Ok(())
///     }
///
///     fn encode_tree<W: Write + Send>(&self, _tree: &Tree, _writer: &mut W) -> Result<(), VctrlError> {
///         // Mock implementation
///         Ok(())
///     }
///
///     fn encode_commit<W: Write + Send>(&self, _commit: &Commit, _writer: &mut W) -> Result<(), VctrlError> {
///         // Mock implementation
///         Ok(())
///     }
///
///     fn encode_tag<W: Write + Send>(&self, _tag: &Tag, _writer: &mut W) -> Result<(), VctrlError> {
///         // Mock implementation
///         Ok(())
///     }
/// }
///
/// let encoder = MockEncoder;
/// let blob = Blob::new(b"file content".to_vec())?;
/// let mut buffer = Vec::new();
/// encoder.encode_blob(&blob, &mut buffer)?;
/// assert_eq!(&buffer, b"file content");
/// # Ok::<(), VctrlError>(())
/// ```
pub trait Encoder: Send + Sync {
    /// Encodes a blob object into a writer.
    ///
    /// # How it works
    /// Writes the raw byte content of the [`Blob`] directly to the provided writer.
    /// Because [`Blob`] enforces size limits during construction, this method does
    /// not need to re-validate the payload size, allowing for a high-throughput,
    /// direct memory-to-stream copy.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if encoding fails. This typically occurs if the underlying
    /// writer experiences an I/O error (e.g., disk full, broken pipe).
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::encoder::Encoder;
    /// # use libvctrl_handler::{Blob, Commit, Tag, Tree, VctrlError};
    /// # use std::io::Write;
    /// # struct MockEncoder;
    /// # impl Encoder for MockEncoder {
    /// #     fn encode_blob<W: Write + Send>(&self, blob: &Blob, writer: &mut W) -> Result<(), VctrlError> { writer.write_all(blob.data())?; Ok(()) }
    /// #     fn encode_tree<W: Write + Send>(&self, _tree: &Tree, _writer: &mut W) -> Result<(), VctrlError> { Ok(()) }
    /// #     fn encode_commit<W: Write + Send>(&self, _commit: &Commit, _writer: &mut W) -> Result<(), VctrlError> { Ok(()) }
    /// #     fn encode_tag<W: Write + Send>(&self, _tag: &Tag, _writer: &mut W) -> Result<(), VctrlError> { Ok(()) }
    /// # }
    /// let encoder = MockEncoder;
    /// let blob = Blob::new(b"binary data".to_vec())?;
    /// let mut buffer = Vec::new();
    /// assert!(encoder.encode_blob(&blob, &mut buffer).is_ok());
    /// # Ok::<(), VctrlError>(())
    /// ```
    fn encode_blob<W: Write + Send>(&self, blob: &Blob, writer: &mut W) -> Result<(), VctrlError>;

    /// Encodes a tree object into a writer.
    ///
    /// # How it works
    /// Serializes the tree entries into the canonical Git binary format. It writes the
    /// mode bits (as octal ASCII), a null byte, the entry name, and the 64-byte SHA-512
    /// hash for each entry. Entries are guaranteed to be in Git-sorted order, as enforced
    /// by the [`Tree`] constructor, ensuring the output is deterministic and canonical.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the underlying writer fails.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::encoder::Encoder;
    /// # use libvctrl_handler::{Blob, Commit, Tag, Tree, VctrlError};
    /// # use std::io::Write;
    /// # struct MockEncoder;
    /// # impl Encoder for MockEncoder {
    /// #     fn encode_blob<W: Write + Send>(&self, _blob: &Blob, _writer: &mut W) -> Result<(), VctrlError> { Ok(()) }
    /// #     fn encode_tree<W: Write + Send>(&self, _tree: &Tree, _writer: &mut W) -> Result<(), VctrlError> { Ok(()) }
    /// #     fn encode_commit<W: Write + Send>(&self, _commit: &Commit, _writer: &mut W) -> Result<(), VctrlError> { Ok(()) }
    /// #     fn encode_tag<W: Write + Send>(&self, _tag: &Tag, _writer: &mut W) -> Result<(), VctrlError> { Ok(()) }
    /// # }
    /// let encoder = MockEncoder;
    /// let tree = Tree::new(vec![])?;
    /// let mut buffer = Vec::new();
    /// assert!(encoder.encode_tree(&tree, &mut buffer).is_ok());
    /// # Ok::<(), VctrlError>(())
    /// ```
    fn encode_tree<W: Write + Send>(&self, tree: &Tree, writer: &mut W) -> Result<(), VctrlError>;

    /// Encodes a commit object into a writer.
    ///
    /// # How it works
    /// Formats the commit into the canonical Git text format. It writes tree references,
    /// parent hashes, author/committer metadata (with timestamps and timezone offsets),
    /// and the commit message. The formatting adheres strictly to Git specifications to
    /// ensure interoperability with standard Git clients.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the underlying writer fails.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::encoder::Encoder;
    /// # use libvctrl_handler::{Blob, Commit, CommitMeta, Hash, Tag, Tree, UserID, VctrlError};
    /// # use std::io::Write;
    /// # struct MockEncoder;
    /// # impl Encoder for MockEncoder {
    /// #     fn encode_blob<W: Write + Send>(&self, _blob: &Blob, _writer: &mut W) -> Result<(), VctrlError> { Ok(()) }
    /// #     fn encode_tree<W: Write + Send>(&self, _tree: &Tree, _writer: &mut W) -> Result<(), VctrlError> { Ok(()) }
    /// #     fn encode_commit<W: Write + Send>(&self, _commit: &Commit, _writer: &mut W) -> Result<(), VctrlError> { Ok(()) }
    /// #     fn encode_tag<W: Write + Send>(&self, _tag: &Tag, _writer: &mut W) -> Result<(), VctrlError> { Ok(()) }
    /// # }
    /// # let hash = Hash::from_bytes(&[0u8; 64])?;
    /// # let user = UserID::new("Alice".to_string(), "alice@example.com".to_string())?;
    /// let encoder = MockEncoder;
    /// let commit = Commit::new(hash, vec![], user, user, "message".to_string())?;
    /// let mut buffer = Vec::new();
    /// assert!(encoder.encode_commit(&commit, &mut buffer).is_ok());
    /// # Ok::<(), VctrlError>(())
    /// ```
    fn encode_commit<W: Write + Send>(
        &self,
        commit: &Commit,
        writer: &mut W,
    ) -> Result<(), VctrlError>;

    /// Encodes a tag object into a writer.
    ///
    /// # How it works
    /// Formats the annotated tag into the canonical Git text format. It writes the target
    /// object hash, tagger identity, and tag message. As with [`encode_commit`](Self::encode_commit),
    /// strict adherence to the Git specification ensures that the resulting tag is recognized
    /// by standard Git tooling.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the underlying writer fails.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::encoder::Encoder;
    /// # use libvctrl_handler::{Blob, Commit, Hash, Tag, Tree, UserID, VctrlError};
    /// # use std::io::Write;
    /// # struct MockEncoder;
    /// # impl Encoder for MockEncoder {
    /// #     fn encode_blob<W: Write + Send>(&self, _blob: &Blob, _writer: &mut W) -> Result<(), VctrlError> { Ok(()) }
    /// #     fn encode_tree<W: Write + Send>(&self, _tree: &Tree, _writer: &mut W) -> Result<(), VctrlError> { Ok(()) }
    /// #     fn encode_commit<W: Write + Send>(&self, _commit: &Commit, _writer: &mut W) -> Result<(), VctrlError> { Ok(()) }
    /// #     fn encode_tag<W: Write + Send>(&self, _tag: &Tag, _writer: &mut W) -> Result<(), VctrlError> { Ok(()) }
    /// # }
    /// # let hash = Hash::from_bytes(&[0u8; 64])?;
    /// # let user = UserID::new("Alice".to_string(), "alice@example.com".to_string())?;
    /// let encoder = MockEncoder;
    /// let tag = Tag::new("v1.0".to_string(), hash, Some(user), "release".to_string())?;
    /// let mut buffer = Vec::new();
    /// assert!(encoder.encode_tag(&tag, &mut buffer).is_ok());
    /// # Ok::<(), VctrlError>(())
    /// ```
    fn encode_tag<W: Write + Send>(&self, tag: &Tag, writer: &mut W) -> Result<(), VctrlError>;
}
