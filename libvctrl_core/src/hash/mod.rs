//! Cryptographic hashing implementations for `libvctrl_core`.
//!
//! # Purpose
//!
//! This module provides concrete, ready-to-use implementations of the
//! [`Hasher`](libvctrl_handler::Hasher) trait. These hashers are used to
//! compute the content-addressable [`Hash`](libvctrl_handler::Hash) values
//! that uniquely identify objects in the version control system.
//!
//! # Design Rationale
//!
//! Hashing is a foundational operation in a content-addressable version
//! control system. Objects are stored and retrieved by the hash of their
//! serialized bytes, so the chosen hash algorithm must be:
//!
//! - **Cryptographically secure**: Collisions must be practically
//!   impossible to prevent an attacker from replacing one object with
//!   another that hashes to the same address.
//! - **Deterministic**: The same input must always produce the same output,
//!   regardless of platform or execution context.
//! - **Efficient**: Hashing large objects should be reasonably fast and
//!   should not require excessive memory or state.
//!
//! By isolating hash implementations in a dedicated module, the core system
//! allows developers to swap cryptographic backends (e.g., from SHA-512 to
//! BLAKE3 or SHA-256) without altering the domain logic. Each hashing
//! algorithm is placed in its own submodule to keep the namespace clean and
//! group related dependencies together.
//!
//! # Internal Mechanism
//!
//! The module currently delegates to pure-Rust, audited crates (like
//! `libvctrl_sha512`) to perform the actual heavy lifting. The structs
//! defined here act as zero-sized adapters that translate the output of
//! those external crates into the canonical
//! [`Hash`](libvctrl_handler::Hash) type expected by `libvctrl_handler`.
//! This adapter pattern keeps the implementation thin and focused on
//! bridging the crate boundaries.
//!
//! # Module Structure
//!
//! The module is organized as follows:
//!
//! - [`sha512`](self::sha512): Contains the [`Sha512Hasher`] struct and its
//!   implementation of [`Hasher`](libvctrl_handler::Hasher).
//! - The root re-exports [`Sha512Hasher`] for ergonomic access.
//!
//! # Security Considerations
//!
//! SHA-512 is chosen for its wide digest (64 bytes), which provides a
//! massive keyspace and resistance to collision attacks. The output length
//! matches [`HASH_LENGTH`](libvctrl_handler::HASH_LENGTH) exactly, avoiding
//! the need for truncation or extension.
//!
//! The delegate crate `libvctrl_sha512` is a pure-Rust implementation that
//! is audited and tested against standard test vectors. No `unsafe` code is
//! used in this crate, preserving the safety guarantees of `libvctrl_core`.
//!
//! # Examples
//!
//! Hashing data with the default SHA-512 hasher:
//!
//! ```
//! use libvctrl_handler::Hasher;
//! use libvctrl_core::hash::Sha512Hasher;
//!
//! let hasher = Sha512Hasher;
//! let hash = hasher.hash(b"data").unwrap();
//! assert_eq!(hash.as_bytes().len(), 64);
//! ```

/// Module containing the [`Sha512Hasher`](crate::hash::Sha512Hasher)
/// implementation.
///
/// # Purpose
///
/// Provides the SHA-512 cryptographic hashing logic, bridging the
/// `libvctrl_sha512` crate with the `libvctrl_handler` contracts.
///
/// # Design Rationale
///
/// SHA-512 is isolated in its own module to ensure that if alternative
/// algorithms (like SHA-256 or BLAKE3) are added in the future, they can be
/// feature-gated and maintained independently without cluttering the global
/// namespace. This separation also keeps the dependency graph explicit: the
/// `sha512` module is the only place where the `libvctrl_sha512` crate is
/// referenced.
///
/// # Internal Mechanism
///
/// The module defines a zero-sized struct `Sha512Hasher` that implements the
/// [`Hasher`](libvctrl_handler::Hasher) trait. The implementation calls the
/// one-shot SHA-512 function provided by `libvctrl_sha512`, obtains a
/// 64-byte digest, and wraps it in a
/// [`Hash`](libvctrl_handler::Hash). The conversion is infallible because
/// SHA-512 always produces exactly 64 bytes, matching
/// [`HASH_LENGTH`](libvctrl_handler::HASH_LENGTH).
///
/// # Examples
///
/// Importing the hasher directly from the submodule:
///
/// ```
/// use libvctrl_handler::Hasher;
/// use libvctrl_core::hash::sha512::Sha512Hasher;
///
/// let hasher = Sha512Hasher;
/// let hash = hasher.hash(b"data").unwrap();
/// assert_eq!(hash.as_bytes().len(), 64);
/// ```
pub mod sha512;

/// Re-export of the [`Sha512Hasher`](crate::hash::sha512::Sha512Hasher)
/// struct.
///
/// # Purpose
///
/// Flattens the module path so users can simply import
/// `libvctrl_core::hash::Sha512Hasher` instead of the full path
/// `libvctrl_core::hash::sha512::Sha512Hasher`.
///
/// # Design Rationale
///
/// Re-exporting at the module root reduces boilerplate and improves the
/// ergonomic experience for consumers of the crate. This pattern is used
/// consistently throughout the crate to maintain a clean public API while
/// preserving internal modularity.
///
/// # Examples
///
/// Using the root re-export:
///
/// ```
/// use libvctrl_handler::Hasher;
/// use libvctrl_core::hash::Sha512Hasher;
///
/// let hasher = Sha512Hasher;
/// let hash = hasher.hash(b"data").unwrap();
/// assert_eq!(hash.as_bytes().len(), 64);
/// ```
pub use sha512::Sha512Hasher;
