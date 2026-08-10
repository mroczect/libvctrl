//! Binary serialization and deserialization for `libvctrl_core`.
//!
//! # Purpose
//! This module provides a concrete implementation of the
//! [`Encoder`](libvctrl_handler::Encoder) and [`Decoder`](libvctrl_handler::Decoder)
//! traits. It translates in-memory version control objects into a compact,
//! deterministic binary format and back.
//!
//! # Design rationale
//! The binary format is designed to be both space-efficient and fast to parse.
//! It uses little-endian integers and length-prefixed byte slices to avoid
//! expensive delimiter scanning. The encoder and decoder are separated into
//! distinct modules to isolate the reading and writing logic, making the code
//! easier to audit and maintain.
//!
//! # Internal mechanism
//! The [`BinaryEncoder`] pre-allocates output buffers based on the estimated
//! size of the object to minimize heap reallocations. The [`BinaryDecoder`]
//! performs strict bounds checking to prevent panics on malformed or truncated
//! data, returning [`VctrlError::CorruptedData`](libvctrl_handler::VctrlError::CorruptedData)
//! when structural invariants are violated.
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

/// Module containing the [`BinaryDecoder`](crate::codec::BinaryDecoder) implementation.
///
/// # Purpose
/// Handles the deserialization of version control objects from the binary wire format.
///
/// # Design rationale
/// The decoder is isolated in its own module to encapsulate the complex,
/// stateful parsing logic (cursor management and bounds checking) separate
/// from the encoding logic.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::{Blob, Encoder, Decoder};
/// use libvctrl_core::codec::binary_decoder::BinaryDecoder;
/// use libvctrl_core::codec::binary_encoder::BinaryEncoder;
///
/// let blob = Blob::new(vec![0u8]);
/// let bytes = BinaryEncoder.encode_blob(&blob).unwrap();
/// let decoded = BinaryDecoder.decode_blob(&bytes).unwrap();
/// assert_eq!(decoded, blob);
/// ```
pub mod binary_decoder;

/// Module containing the [`BinaryEncoder`](crate::codec::BinaryEncoder) implementation.
///
/// # Purpose
/// Handles the serialization of version control objects into the binary wire format.
///
/// # Design rationale
/// The encoder is isolated in its own module to group all the serialization
/// and byte-allocation logic together, ensuring that changes to the wire format
/// only affect this specific part of the codebase.
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

/// Re-export of the [`BinaryDecoder`](crate::codec::binary_decoder::BinaryDecoder) struct.
///
/// # Purpose
/// Provides convenient access to the decoder at the module root level without
/// requiring the caller to navigate the internal module hierarchy.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::{Blob, Encoder, Decoder};
/// use libvctrl_core::codec::{BinaryEncoder, BinaryDecoder};
///
/// let blob = Blob::new(vec![1, 2, 3]);
/// let bytes = BinaryEncoder.encode_blob(&blob).unwrap();
/// let decoder = BinaryDecoder;
/// assert!(decoder.decode_blob(&bytes).is_ok());
/// ```
pub use binary_decoder::BinaryDecoder;

/// Re-export of the [`BinaryEncoder`](crate::codec::binary_encoder::BinaryEncoder) struct.
///
/// # Purpose
/// Provides convenient access to the encoder at the module root level without
/// requiring the caller to navigate the internal module hierarchy.
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
