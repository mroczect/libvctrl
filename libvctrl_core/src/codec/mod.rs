//! Binary serialization and deserialization for version control objects.
//!
//! This module provides a pair of reference implementations:
//! [`BinaryEncoder`] and [`BinaryDecoder`].
//!
//! **Stability note:** The binary format used here is a *reference format*
//! and is **not covered by semantic versioning guarantees**. It may change
//! between minor releases. For production use, build your own encoder/decoder
//! or pin to a specific version of this crate after verifying compatibility.

pub mod binary_decoder;
pub mod binary_encoder;

pub use binary_decoder::BinaryDecoder;
pub use binary_encoder::BinaryEncoder;
