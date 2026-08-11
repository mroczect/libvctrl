//! Cryptographic hashing implementations for `libvctrl_core`.
//!
//! # Purpose
//! This module provides concrete, ready-to-use implementations of the
//! [`Hasher`](libvctrl_handler::Hasher) trait. These hashers are used to
//! compute the content-addressable [`Hash`](libvctrl_handler::Hash) values
//! that uniquely identify objects in the version control system.
//!
//! # Design rationale
//! By isolating hash implementations in a dedicated module, the core system
//! allows developers to swap cryptographic backends (e.g., from SHA-512 to
//! BLAKE3) without altering the domain logic. Each hashing algorithm is
//! placed in its own submodule to keep the namespace clean and group related
//! dependencies together.
//!
//! # Internal mechanism
//! The module currently delegates to pure-Rust, audited crates (like
//! `libvctrl_sha512`) to perform the actual heavy lifting. The structs
//! defined here act as zero-sized adapters that translate the output of
//! those external crates into the canonical [`Hash`](libvctrl_handler::Hash)
//! type expected by `libvctrl_handler`.

/// Module containing the [`Sha512Hasher`](crate::hash::Sha512Hasher) implementation.
///
/// # Purpose
/// Provides the SHA-512 cryptographic hashing logic, bridging the
/// `libvctrl_sha512` crate with the `libvctrl_handler` contracts.
///
/// # Design rationale
/// SHA-512 is isolated in its own module to ensure that if alternative
/// algorithms (like SHA-256 or BLAKE3) are added in the future, they can
/// be feature-gated and maintained independently without cluttering the
/// global namespace.
///
/// # Examples
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

/// Re-export of the [`Sha512Hasher`](crate::hash::sha512::Sha512Hasher) struct.
///
/// # Purpose
/// Flattens the module path so users can simply import
/// `libvctrl_core::hash::Sha512Hasher` instead of the full path.
///
/// # Design rationale
/// Re-exporting at the module root reduces boilerplate and improves the
/// ergonomic experience for consumers of the crate.
///
/// # Examples
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
