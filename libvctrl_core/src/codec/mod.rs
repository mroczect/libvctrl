//! Binary serialization and deserialization for `libvctrl_core`.
//!
//! # Purpose
//!
//! This module provides a concrete implementation of the
//! [`Encoder`](libvctrl_handler::Encoder) and
//! [`Decoder`](libvctrl_handler::Decoder) traits. It translates in-memory
//! version control objects ([`Blob`](libvctrl_handler::Blob),
//! [`Tree`](libvctrl_handler::Tree), [`Commit`](libvctrl_handler::Commit),
//! [`Tag`](libvctrl_handler::Tag)) into a compact, deterministic binary
//! format and back.
//!
//! # Design Rationale
//!
//! The binary format is designed to be both space-efficient and fast to
//! parse. It uses little-endian integers and length-prefixed byte slices to
//! avoid expensive delimiter scanning. The encoder and decoder are separated
//! into distinct modules to isolate the reading and writing logic, making the
//! code easier to audit and maintain.
//!
//! ## Versioning
//!
//! Every serialized object begins with a single version byte. This allows
//! the format to evolve over time. A decoder that encounters an unsupported
//! version can reject the payload cleanly rather than attempting to parse
//! incompatible data.
//!
//! ## Endianness
//!
//! All integer fields are encoded in little-endian byte order. This matches
//! the native byte order of most modern CPU architectures (x86, x86_64,
//! ARM little-endian), which minimizes byte-swapping overhead during
//! encoding and decoding.
//!
//! ## Length-Prefixed Fields
//!
//! Variable-length fields such as names, email addresses, messages, and
//! blob data are prefixed with their length. This allows the decoder to
//! know exactly how many bytes to read and enables pre-allocation of buffers.
//! Length prefixes also provide a simple structural validation point: if the
//! declared length does not match the remaining data, the payload is corrupt.
//!
//! # Internal Mechanism
//!
//! The [`BinaryEncoder`] pre-allocates output buffers based on the estimated
//! size of the object to minimize heap reallocations. It then appends
//! fields sequentially using `extend_from_slice` for efficient bulk copies.
//!
//! The [`BinaryDecoder`] performs strict bounds checking on every slice
//! access. It maintains a cursor over the input and never reads past the
//! end of the buffer. If the data is truncated, malformed, or contains
//! invalid UTF-8, the decoder returns
//! [`VctrlError::CorruptedData`](libvctrl_handler::VctrlError::CorruptedData)
//! rather than panicking.
//!
//! # Security Considerations
//!
//! - **Panic-free decoding**: The decoder is designed to be completely
//!   panic-free for arbitrary input. All accesses are bounds-checked.
//! - **DoS protection**: Before allocating memory for variable-length
//!   fields, the decoder validates declared lengths against system limits
//!   such as
//!   [`MAX_BLOB_SIZE`](libvctrl_handler::MAX_BLOB_SIZE),
//!   [`MAX_MESSAGE_LENGTH`](libvctrl_handler::MAX_MESSAGE_LENGTH), and
//!   [`MAX_TREE_ENTRIES`](libvctrl_handler::MAX_TREE_ENTRIES). This
//!   prevents a malicious payload from requesting a huge allocation.
//! - **Strict UTF-8 validation**: All string fields are validated using
//!   [`std::str::from_utf8`]. Invalid sequences are rejected.
//!
//! # Round-Trip Guarantee
//!
//! For every object type supported by this codec, encoding an object with
//! [`BinaryEncoder`] and then decoding it with [`BinaryDecoder`] yields a
//! value equal to the original object. This property is tested extensively
//! with unit tests and property-based tests using `proptest`.
//!
//! # Examples
//!
//! Performing a full encode-decode round-trip on a `Blob`:
//!
//! ```
//! use libvctrl_handler::{Blob, Decoder, Encoder};
//! use libvctrl_core::codec::{BinaryDecoder, BinaryEncoder};
//!
//! let original_blob = Blob::new(b"hello world".to_vec());
//! let encoder = BinaryEncoder;
//! let decoder = BinaryDecoder;
//!
//! let encoded_bytes = encoder.encode_blob(&original_blob).unwrap();
//! let decoded_blob = decoder.decode_blob(&encoded_bytes).unwrap();
//!
//! assert_eq!(decoded_blob, original_blob);
//! ```

/// Module containing the [`BinaryDecoder`](crate::codec::BinaryDecoder)
/// implementation.
///
/// # Purpose
///
/// Handles the deserialization of version control objects from the binary
/// wire format.
///
/// # Design Rationale
///
/// The decoder is isolated in its own module to encapsulate the complex,
/// stateful parsing logic (cursor management and bounds checking) separate
/// from the encoding logic. This separation makes the code easier to audit
/// because all risky slice operations are confined to one file.
///
/// # Internal Mechanism
///
/// The decoder maintains a cursor index into the input byte slice. It reads
/// length prefixes, advances the cursor, and extracts sub-slices. Before
/// each access, it checks that the requested range is within bounds. This
/// guarantees that malformed input never causes an out-of-bounds panic.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::{Blob, Decoder, Encoder};
/// use libvctrl_core::codec::binary_decoder::BinaryDecoder;
/// use libvctrl_core::codec::binary_encoder::BinaryEncoder;
///
/// let blob = Blob::new(vec![0u8]);
/// let bytes = BinaryEncoder.encode_blob(&blob).unwrap();
/// let decoded = BinaryDecoder.decode_blob(&bytes).unwrap();
/// assert_eq!(decoded, blob);
/// ```
pub mod binary_decoder;

/// Module containing the [`BinaryEncoder`](crate::codec::BinaryEncoder)
/// implementation.
///
/// # Purpose
///
/// Handles the serialization of version control objects into the binary wire
/// format.
///
/// # Design Rationale
///
/// The encoder is isolated in its own module to group all the serialization
/// and byte-allocation logic together, ensuring that changes to the wire
/// format only affect this specific part of the codebase. The encoder is
/// stateless and deterministic; encoding the same object always produces the
/// same byte sequence.
///
/// # Internal Mechanism
///
/// The encoder pre-allocates a `Vec<u8>` based on the estimated size of the
/// object, reducing the number of heap reallocations. It writes fields using
/// `extend_from_slice`, which compiles to efficient `memcpy` operations for
/// bulk data.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::{Blob, Encoder};
/// use libvctrl_core::codec::binary_encoder::BinaryEncoder;
///
/// let blob = Blob::new(vec![0u8]);
/// let bytes = BinaryEncoder.encode_blob(&blob).unwrap();
/// assert!(!bytes.is_empty());
/// ```
pub mod binary_encoder;

/// Re-export of the [`BinaryDecoder`](crate::codec::binary_decoder::BinaryDecoder)
/// struct.
///
/// # Purpose
///
/// Provides convenient access to the decoder at the module root level without
/// requiring the caller to navigate the internal module hierarchy. This keeps
/// the public API surface clean and intuitive.
///
/// # Design Rationale
///
/// Re-exporting at the module root is a common Rust pattern. It allows users
/// to write `use libvctrl_core::codec::BinaryDecoder;` instead of the longer
/// `use libvctrl_core::codec::binary_decoder::BinaryDecoder;`.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::{Blob, Decoder, Encoder};
/// use libvctrl_core::codec::{BinaryDecoder, BinaryEncoder};
///
/// let blob = Blob::new(vec![1, 2, 3]);
/// let bytes = BinaryEncoder.encode_blob(&blob).unwrap();
/// let decoder = BinaryDecoder;
/// assert!(decoder.decode_blob(&bytes).is_ok());
/// ```
pub use binary_decoder::BinaryDecoder;

/// Re-export of the [`BinaryEncoder`](crate::codec::binary_encoder::BinaryEncoder)
/// struct.
///
/// # Purpose
///
/// Provides convenient access to the encoder at the module root level without
/// requiring the caller to navigate the internal module hierarchy. This keeps
/// the public API surface clean and intuitive.
///
/// # Design Rationale
///
/// Re-exporting at the module root simplifies imports and aligns with the
/// pattern used throughout the crate. It also ensures that if the internal
/// module layout changes, downstream code that uses the root re-export will
/// not break.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::{Blob, Encoder};
/// use libvctrl_core::codec::BinaryEncoder;
///
/// let encoder = BinaryEncoder;
/// let blob = Blob::new(vec![1, 2, 3]);
/// assert!(encoder.encode_blob(&blob).is_ok());
/// ```
pub use binary_encoder::BinaryEncoder;
