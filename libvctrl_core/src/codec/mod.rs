//! Binary serialization and deserialization for version control objects.
//!
//! This module provides a pair of reference implementations:
//! [`BinaryEncoder`] and [`BinaryDecoder`].
//!
//! They operate on a simple, deterministic binary format that is easy
//! to inspect, debug, and replace with custom implementations.

pub mod binary_decoder;
pub mod binary_encoder;

pub use binary_decoder::BinaryDecoder;
pub use binary_encoder::BinaryEncoder;
