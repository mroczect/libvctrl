//! Binary serialization and deserialization for version control objects.
//!
//! This module provides the reference implementation of the [`Encoder`] and
//! [`Decoder`] traits defined in `libvctrl_handler`. It defines a simple,
//! deterministic binary format that covers all four core object types:
//! [`Blob`], [`Tree`], [`Commit`], and [`Tag`].
//!
//! # Purpose
//!
//! The primary goal of this module is to **prove** that the abstract contracts
//! can be fulfilled with a concrete implementation. It is intentionally
//! minimal: no compression, no indexing, no streaming – just straightforward
//! serialization and deserialization that is easy to audit and understand.
//!
//! # Format overview
//!
//! Every object is encoded as a sequence of bytes with no framing other than
//! length prefixes. The format is **not self-describing**; you must know the
//! object type before decoding. This is consistent with content‑addressed
//! storage, where the hash identifies both the object and (implicitly) its
//! type.
//!
//! ## Blob
//! - 8‑byte little‑endian data length
//! - raw data bytes
//!
//! ## Tree
//! - 4‑byte little‑endian entry count
//! - for each entry:
//!     - 1‑byte name length
//!     - name bytes (UTF‑8)
//!     - 1‑byte entry kind (`0` = Blob, `1` = Tree)
//!     - 64‑byte hash
//!
//! ## Commit
//! - 64‑byte tree hash
//! - 1‑byte parent count
//! - for each parent: 64‑byte hash
//! - 1‑byte author name length + name bytes
//! - 1‑byte author email length + email bytes
//! - 1‑byte committer name length + name bytes
//! - 1‑byte committer email length + email bytes
//! - 4‑byte little‑endian message length + message bytes (UTF‑8)
//!
//! ## Tag
//! - 1‑byte name length + name bytes (UTF‑8)
//! - 64‑byte target hash
//! - 1‑byte tagger presence flag (`0` or `1`)
//! - if flag is `1`:
//!     - 1‑byte tagger name length + name bytes
//!     - 1‑byte tagger email length + email bytes
//! - 4‑byte little‑endian message length + message bytes (UTF‑8)
//!
//! # `DoS` prevention
//!
//! The decoder enforces the limits defined in `libvctrl_handler::constants`:
//! [`MAX_BLOB_SIZE`], [`MAX_TREE_ENTRIES`], [`MAX_MESSAGE_LENGTH`].
//! Inputs exceeding these limits are rejected with [`VctrlError::CorruptedData`].
//!
//! # Round‑trip guarantee
//!
//! For any valid object, encoding followed by decoding yields the original
//! object (modulo byte‑level equality of data). This property is tested in
//! the crate's integration tests.
//!
//! # Stability note
//!
//! The binary format described here is a **reference format** and is **not**
//! covered by semantic versioning guarantees. It may change between minor
//! releases. For production use, either pin to a specific version of this
//! crate or implement your own encoder/decoder pair that meets your stability
//! requirements.
//!
//! # Usage
//!
//! ```rust
//! use libvctrl_core::codec::{BinaryEncoder, BinaryDecoder};
//! use libvctrl_handler::{Blob, Encoder, Decoder};
//!
//! let blob = Blob::new(b"Hello, world!".to_vec());
//! let encoded = BinaryEncoder.encode_blob(&blob).unwrap();
//! let decoded = BinaryDecoder.decode_blob(&encoded).unwrap();
//! assert_eq!(decoded.data(), blob.data());
//! ```

pub mod binary_decoder;
pub mod binary_encoder;

pub use binary_decoder::BinaryDecoder;
pub use binary_encoder::BinaryEncoder;
