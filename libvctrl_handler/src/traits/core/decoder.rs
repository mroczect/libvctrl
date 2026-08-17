//! Object decoder trait.
//!
//! # Architecture
//! This module defines the contract for deserializing raw byte streams into
//! strongly-typed Git domain objects ([`Blob`], [`Tree`], [`Commit`], [`Tag`]).
//! It acts as the bridge between unstructured I/O data and the crate's type-safe
//! in-memory representations.
//!
//! # Design Rationale: Streaming Deserialization
//! Instead of accepting a `&[u8]` or `Vec<u8>`, the decoder methods require a
//! generic `R: Read` bound. This is a critical architectural decision: it forces
//! streaming deserialization. Git objects (especially blobs) can be massive.
//! By reading from a stream, the decoder can process gigabytes of data with a
//! fixed memory footprint, preventing denial-of-service (DoS) vulnerabilities
//! associated with unbounded memory allocation.

use crate::errors::VctrlError;
use crate::types::{Blob, Commit, Tag, Tree};
use std::io::Read;

/// Trait for decoding raw Git object bytes into structured types.
///
/// # Why this exists
/// Abstracts the parsing logic away from the storage backend. Whether objects
/// are being read from loose files on disk, extracted from a compressed packfile,
/// or streamed over a network socket, the decoding logic remains identical.
/// This allows the crate to support multiple wire formats or compression
/// algorithms by simply providing different implementations of this trait.
///
/// # How it works
/// The trait uses generic methods (`<R: Read + Send>`) rather than dynamic
/// trait objects (`&mut dyn Read`). This design leverages Rust's monomorphization:
/// the compiler generates a specific version of the decode function for every
/// concrete reader type used at runtime. This eliminates dynamic dispatch overhead,
/// allowing the compiler to aggressively inline the reading logic.
///
/// # Design Rationale: Thread Safety
/// The trait requires `Send + Sync` on `Self`, and `Send` on the reader `R`.
/// This ensures that decoding operations can be safely dispatched to a thread pool.
/// For example, when parsing a multi-object packfile, the engine can distribute
/// object streams across multiple worker threads to utilize multi-core parallelism
/// without risking data races.
///
/// # Examples
///
/// Implementing the trait for a mock streaming parser:
///
/// ```
/// # use libvctrl_handler::traits::core::decoder::Decoder;
/// # use libvctrl_handler::{Blob, Commit, Tag, Tree, VctrlError};
/// # use std::io::{Cursor, Read};
/// #
/// struct MockDecoder;
///
/// impl Decoder for MockDecoder {
///     fn decode_blob<R: Read + Send>(&self, mut reader: R) -> Result<Blob, VctrlError> {
///         let mut buf = Vec::new();
///         reader.read_to_end(&mut buf)?;
///         Blob::new(buf)
///     }
///
///     fn decode_tree<R: Read + Send>(&self, _reader: R) -> Result<Tree, VctrlError> {
///         // Mock implementation returns an empty tree
///         Tree::new(vec![])
///     }
///
///     fn decode_commit<R: Read + Send>(&self, _reader: R) -> Result<Commit, VctrlError> {
///         // Mock implementation returns an error for brevity
///         Err(VctrlError::Other("mock commit decode".into()))
///     }
///
///     fn decode_tag<R: Read + Send>(&self, _reader: R) -> Result<Tag, VctrlError> {
///         Err(VctrlError::Other("mock tag decode".into()))
///     }
/// }
///
/// let decoder = MockDecoder;
/// let raw_data = Cursor::new(b"file content".to_vec());
/// let blob = decoder.decode_blob(raw_data)?;
/// assert_eq!(blob.data(), b"file content");
/// # Ok::<(), VctrlError>(())
/// ```
pub trait Decoder: Send + Sync {
    /// Decodes a blob object from a reader.
    ///
    /// # How it works
    /// Reads bytes from the provided reader until EOF, enforcing the
    /// [`MAX_BLOB_SIZE`](crate::constants::MAX_BLOB_SIZE) limit during the
    /// construction of the [`Blob`] type. This prevents memory exhaustion
    /// from maliciously large streams.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if decoding fails. This can occur if the reader
    /// encounters an I/O error, or if the parsed data exceeds the maximum
    /// allowed size limits.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::decoder::Decoder;
    /// # use libvctrl_handler::{Blob, Commit, Tag, Tree, VctrlError};
    /// # use std::io::{Cursor, Read};
    /// #
    /// # struct MockDecoder;
    /// # impl Decoder for MockDecoder {
    /// #     fn decode_blob<R: Read + Send>(&self, mut reader: R) -> Result<Blob, VctrlError> {
    /// #         let mut buf = Vec::new();
    /// #         reader.read_to_end(&mut buf)?;
    /// #         Blob::new(buf)
    /// #     }
    /// #     fn decode_tree<R: Read + Send>(&self, _reader: R) -> Result<Tree, VctrlError> { Tree::new(vec![]) }
    /// #     fn decode_commit<R: Read + Send>(&self, _reader: R) -> Result<Commit, VctrlError> { Err(VctrlError::Other("mock".into())) }
    /// #     fn decode_tag<R: Read + Send>(&self, _reader: R) -> Result<Tag, VctrlError> { Err(VctrlError::Other("mock".into())) }
    /// # }
    /// let decoder = MockDecoder;
    /// let stream = Cursor::new(b"binary data".to_vec());
    /// assert!(decoder.decode_blob(stream).is_ok());
    /// ```
    fn decode_blob<R: Read + Send>(&self, reader: R) -> Result<Blob, VctrlError>;

    /// Decodes a tree object from a reader.
    ///
    /// # How it works
    /// Parses the binary tree format, reading entry modes, names, and hashes
    /// sequentially. It enforces Git's strict sorting rules (directories are
    /// sorted as if they have a trailing `/`) and rejects duplicate entries
    /// during the construction of the [`Tree`] type.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if decoding fails. This can occur if the stream
    /// is truncated, contains invalid mode bits, or violates tree structural
    /// integrity (e.g., unsorted entries).
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::decoder::Decoder;
    /// # use libvctrl_handler::{Blob, Commit, Tag, Tree, VctrlError};
    /// # use std::io::{Cursor, Read};
    /// #
    /// # struct MockDecoder;
    /// # impl Decoder for MockDecoder {
    /// #     fn decode_blob<R: Read + Send>(&self, mut reader: R) -> Result<Blob, VctrlError> { Blob::new(Vec::new()) }
    /// #     fn decode_tree<R: Read + Send>(&self, _reader: R) -> Result<Tree, VctrlError> { Tree::new(vec![]) }
    /// #     fn decode_commit<R: Read + Send>(&self, _reader: R) -> Result<Commit, VctrlError> { Err(VctrlError::Other("mock".into())) }
    /// #     fn decode_tag<R: Read + Send>(&self, _reader: R) -> Result<Tag, VctrlError> { Err(VctrlError::Other("mock".into())) }
    /// # }
    /// let decoder = MockDecoder;
    /// let stream = Cursor::new(Vec::new());
    /// assert!(decoder.decode_tree(stream).is_ok());
    /// ```
    fn decode_tree<R: Read + Send>(&self, reader: R) -> Result<Tree, VctrlError>;

    /// Decodes a commit object from a reader.
    ///
    /// # How it works
    /// Parses the textual commit format, extracting tree references, parent
    /// hashes, author/committer metadata, and the commit message. It validates
    /// parent counts and message lengths against crate constants before
    /// constructing the [`Commit`] type.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if decoding fails. This can occur if the commit
    /// contains duplicate parents, if the timestamp is malformed, or if an
    /// I/O error occurs while reading the stream.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::decoder::Decoder;
    /// # use libvctrl_handler::{Blob, Commit, Tag, Tree, VctrlError};
    /// # use std::io::{Cursor, Read};
    /// #
    /// # struct MockDecoder;
    /// # impl Decoder for MockDecoder {
    /// #     fn decode_blob<R: Read + Send>(&self, _reader: R) -> Result<Blob, VctrlError> { Blob::new(Vec::new()) }
    /// #     fn decode_tree<R: Read + Send>(&self, _reader: R) -> Result<Tree, VctrlError> { Tree::new(vec![]) }
    /// #     fn decode_commit<R: Read + Send>(&self, _reader: R) -> Result<Commit, VctrlError> { Err(VctrlError::Other("mock".into())) }
    /// #     fn decode_tag<R: Read + Send>(&self, _reader: R) -> Result<Tag, VctrlError> { Err(VctrlError::Other("mock".into())) }
    /// # }
    /// let decoder = MockDecoder;
    /// let stream = Cursor::new(Vec::new());
    /// assert!(decoder.decode_commit(stream).is_err()); // Mock returns err
    /// ```
    fn decode_commit<R: Read + Send>(&self, reader: R) -> Result<Commit, VctrlError>;

    /// Decodes a tag object from a reader.
    ///
    /// # How it works
    /// Parses the annotated tag format, extracting the target object hash,
    /// tagger identity, and tag message. It enforces reference naming rules
    /// (via [`validate_ref_name`](crate::validation::validate_ref_name)) on the
    /// tag's name during construction.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if decoding fails. This can occur if the tag name
    /// is invalid, if the message exceeds the maximum length, or if the stream
    /// is corrupted.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::decoder::Decoder;
    /// # use libvctrl_handler::{Blob, Commit, Tag, Tree, VctrlError};
    /// # use std::io::{Cursor, Read};
    /// #
    /// # struct MockDecoder;
    /// # impl Decoder for MockDecoder {
    /// #     fn decode_blob<R: Read + Send>(&self, _reader: R) -> Result<Blob, VctrlError> { Blob::new(Vec::new()) }
    /// #     fn decode_tree<R: Read + Send>(&self, _reader: R) -> Result<Tree, VctrlError> { Tree::new(vec![]) }
    /// #     fn decode_commit<R: Read + Send>(&self, _reader: R) -> Result<Commit, VctrlError> { Err(VctrlError::Other("mock".into())) }
    /// #     fn decode_tag<R: Read + Send>(&self, _reader: R) -> Result<Tag, VctrlError> { Err(VctrlError::Other("mock".into())) }
    /// # }
    /// let decoder = MockDecoder;
    /// let stream = Cursor::new(Vec::new());
    /// assert!(decoder.decode_tag(stream).is_err()); // Mock returns err
    /// ```
    fn decode_tag<R: Read + Send>(&self, reader: R) -> Result<Tag, VctrlError>;
}
