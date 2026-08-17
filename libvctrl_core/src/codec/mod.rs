//! # Binary Codec
//!
//! This module provides the reference implementation of the binary
//! serialization format for Git objects. It contains two zero-sized types:
//!
//! - [`BinaryEncoder`](binary_encoder::BinaryEncoder): writes objects into a
//!   deterministic, versioned byte stream.
//! - [`BinaryDecoder`](binary_decoder::BinaryDecoder): reads such byte streams
//!   back into strongly validated, immutable objects.
//!
//! ## Why this module exists
//!
//! Version control systems rely on content addressing. To compute a stable
//! hash, objects must be serialized in a way that is independent of platform,
//! compiler, and runtime conditions. This module defines such a canonical
//! encoding and the corresponding decoding logic.
//!
//! The encoder and decoder are deliberately separate to enforce a clear
//! boundary between producing bytes and consuming untrusted bytes. The decoder
//! performs extensive bounds and validity checks, whereas the encoder assumes
//! its input objects are already valid.
//!
//! ## How it works
//!
//! Every encoded object begins with a single version byte. The current version
//! is [`VERSION`](binary_encoder::VERSION) = 3. The decoder rejects any input
//! whose first byte does not match this value.
//!
//! After the version byte, fields are written in a strict order using
//! little-endian integer encoding. Strings are length-prefixed with a single
//! byte; larger payloads (like blob content or commit messages) use dedicated
//! 32-bit or 64-bit length prefixes.
//!
//! ## Examples
//!
//! The following example shows a complete round-trip through the encoder and
//! decoder. It encodes a [`Blob`], then decodes it back and asserts equality.
//!
//! ```
//! # use std::io::Cursor;
//! # use libvctrl_handler::{Blob, Decoder, Encoder};
//! # use libvctrl_core::codec::{BinaryDecoder, BinaryEncoder};
//! let original = Blob::new(b"round trip".to_vec()).unwrap();
//!
//! let mut encoded = Vec::new();
//! BinaryEncoder.encode_blob(&original, &mut encoded).unwrap();
//!
//! let decoded = BinaryDecoder
//!     .decode_blob(Cursor::new(encoded.as_slice()))
//!     .unwrap();
//!
//! assert_eq!(original, decoded);
//! ```

/// Binary decoder for Git objects.
///
/// This submodule provides [`BinaryDecoder`](self::BinaryDecoder), the
/// strictly validated inverse of the encoder. It accepts any
/// [`std::io::Read`] source and returns either a fully constructed object or a
/// [`VctrlError`] describing the exact corruption encountered.
pub mod binary_decoder;

/// Binary encoder for Git objects.
///
/// This submodule provides [`BinaryEncoder`](self::BinaryEncoder), the
/// canonical producer of binary object data. It writes directly to any
/// [`std::io::Write`] sink without intermediate heap allocations.
pub mod binary_encoder;

pub use binary_decoder::BinaryDecoder;
pub use binary_encoder::BinaryEncoder;
