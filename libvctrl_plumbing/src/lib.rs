//! # libvctrl_plumbing
//!
//! Plumbing commands for the libvctrl version control system.
//!
//! This crate provides low-level commands that operate directly on object
//! stores, references, and codecs. Unlike porcelain commands, plumbing
//! commands expose detailed control and are intended for scripting and for
//! building higher-level commands.
//!
//! ## Why this crate exists
//!
//! Version control systems separate low-level (plumbing) commands from
//! high-level (porcelain) commands. Plumbing commands are stable, composable,
//! and designed for programmatic use. They perform one job well and produce
//! machine-readable output where possible. This crate implements those
//! foundational commands using the unified facade provided by the
//! [`libvctrl`](https://docs.rs/libvctrl) crate.
//!
//! ## Architecture
//!
//! The crate is organized by command modules:
//!
//! - [`cat_file`](crate::cat_file) — inspects object content and metadata by
//!   hash.
//!
//! Additional plumbing commands will follow the same pattern. Each module
//! contains one or more public functions that accept trait objects
//! (for example, `&dyn ObjectStore` and `&dyn Decoder`), making the commands
//! backend-agnostic and independently testable.
//!
//! ## How it works
//!
//! A typical plumbing command:
//!
//! 1. Parses and validates its arguments.
//! 2. Uses an [`ObjectStore`](libvctrl::ObjectStore) to fetch raw bytes.
//! 3. Uses a [`Decoder`](libvctrl::Decoder) to interpret those bytes.
//! 4. Writes the requested result to an output writer.
//!
//! This design allows the same command to run against any storage backend
//! (in-memory, filesystem, remote) and any codec, as long as the appropriate
//! traits are implemented.
//!
//! ## Safety and correctness
//!
//! All commands return [`VctrlError`](libvctrl::VctrlError) on failure and
//! never panic on malformed user input. Output writers are used exclusively
//! through [`std::io::Write`], and all I/O errors are propagated with their
//! original error wrapped in the unified error type.
//!
//! ## Example
//!
//! The following example stores a blob and uses [`cat_file`] to query its
//! type:
//!
//! ```
//! # use libvctrl::{Blob, Encoder, Hasher, ObjectStore, BinaryEncoder, BinaryDecoder, Sha512Hasher, MemoryStore};
//! # use libvctrl_plumbing::{cat_file, CatFileMode};
//! # fn main() -> Result<(), libvctrl::VctrlError> {
//! let blob = Blob::new(b"example".to_vec())?;
//!
//! let mut encoded = Vec::new();
//! BinaryEncoder.encode_blob(&blob, &mut encoded)?;
//! let hash = Sha512Hasher.hash(&mut encoded.as_slice())?;
//!
//! let mut store = MemoryStore::new();
//! store.put(&hash, &encoded)?;
//!
//! let mut out = Vec::new();
//! cat_file(
//!     &store,
//!     &BinaryDecoder,
//!     &hash.to_string(),
//!     CatFileMode::ObjectType,
//!     &mut out,
//! )?;
//!
//! assert_eq!(String::from_utf8(out).unwrap(), "blob\n");
//! # Ok(())
//! # }
//! ```

#[cfg(test)]
use libvctrl_core as _;

/// Plumbing command for inspecting object content and metadata.
///
/// This module implements the `cat-file` command, which retrieves an object by
/// its hash and prints its type, size, pretty-printed content, or raw bytes
/// depending on the requested mode. It also supports batch processing of
/// multiple objects with configurable formatting.
pub mod cat_file;

pub use cat_file::{BatchOptions, CatFileMode, ObjectType, cat_file, cat_file_batch};
