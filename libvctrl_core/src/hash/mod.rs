//! Cryptographic hash function implementations.
//!
//! Currently provides only [`Sha512Hasher`].

pub mod sha512;
pub use sha512::Sha512Hasher;
