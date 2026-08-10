//! Ephemeral in-memory storage backends for `libvctrl_core`.
//!
//! # Purpose
//! This module provides concrete, RAM-resident implementations of the
//! [`ObjectStore`](libvctrl_handler::ObjectStore) and
//! [`RefStore`](libvctrl_handler::RefStore) traits. These backends are designed
//! for testing, caching, and short-lived sessions where persistence to disk or
//! network is unnecessary.
//!
//! # Design rationale
//! - **Ephemeral State**: Data stored in these backends is lost when the process
//!   exits. This makes them ideal for unit tests where isolation and speed are
//!   critical.
//! - **Structural Separation**: Objects (content-addressed) and references
//!   (name-addressed) are kept in distinct stores. This mirrors the design of
//!   persistent version control systems (like Git) where loose objects and
//!   reference files occupy different directory structures.
//! - **HashMap Utilization**: Both stores leverage `std::collections::HashMap`
//!   to achieve average O(1) time complexity for insertions, lookups, and
//!   deletions.
//!
//! # Internal mechanism
//! The [`MemoryStore`] maps a 64-byte [`Hash`](libvctrl_handler::Hash) to a
//! `Vec<u8>` payload, wrapping it in a `std::io::Cursor` for streaming reads.
//! The [`MemoryRefStore`] maps a `String` name to a
//! [`Hash`](libvctrl_handler::Hash). Both structs encapsulate their internal
//! maps as private fields, ensuring that all mutations occur through the
//! trait methods to enforce validation rules (like name length checks).

/// Module containing the [`MemoryStore`](crate::store::MemoryStore) implementation.
///
/// # Purpose
/// Provides an in-memory key-value store for raw version control objects,
/// addressable by their cryptographic hash.
///
/// # Design rationale
/// Encapsulating this logic in its own module isolates the storage mechanics
/// from the trait definitions, making the codebase easier to maintain and test.
///
/// # Examples
///
/// ```
/// use libvctrl_core::store::memory::MemoryStore;
/// use libvctrl_handler::{Hash, ObjectStore};
/// use std::io::Read;
///
/// let mut store = MemoryStore::new();
/// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
/// store.put(&hash, b"object data").unwrap();
///
/// let mut reader = store.get(&hash).unwrap();
/// let mut buf = Vec::new();
/// reader.read_to_end(&mut buf).unwrap();
/// assert_eq!(buf, b"object data");
/// ```
pub mod memory;

/// Module containing the [`MemoryRefStore`](crate::store::MemoryRefStore) implementation.
///
/// # Purpose
/// Provides an in-memory store for named references (e.g., branches, tags)
/// that point to specific object hashes.
///
/// # Design rationale
/// Separating reference storage from object storage allows reference updates
/// to be highly mutable without affecting the immutable object database.
///
/// # Examples
///
/// ```
/// use libvctrl_core::store::ref_store::MemoryRefStore;
/// use libvctrl_handler::{Hash, RefStore};
///
/// let mut store = MemoryRefStore::new();
/// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
/// store.set_ref("refs/heads/main", &hash).unwrap();
/// assert_eq!(store.get_ref("refs/heads/main").unwrap(), hash);
/// ```
pub mod ref_store;

/// Re-export of the [`MemoryStore`](crate::store::memory::MemoryStore) struct.
///
/// # Purpose
/// Flattens the module path so users can simply import
/// `libvctrl_core::store::MemoryStore` instead of the full internal path.
///
/// # Design rationale
/// Re-exporting at the module root reduces boilerplate and improves the
/// ergonomic experience for consumers of the crate.
///
/// # Examples
///
/// ```
/// use libvctrl_core::store::MemoryStore;
/// use libvctrl_handler::{Hash, ObjectStore};
/// use std::io::Read;
///
/// let mut store = MemoryStore::new();
/// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
/// store.put(&hash, b"data").unwrap();
///
/// let mut buf = Vec::new();
/// store.get(&hash).unwrap().read_to_end(&mut buf).unwrap();
/// assert_eq!(buf, b"data");
/// ```
pub use memory::MemoryStore;

/// Re-export of the [`MemoryRefStore`](crate::store::ref_store::MemoryRefStore) struct.
///
/// # Purpose
/// Flattens the module path so users can simply import
/// `libvctrl_core::store::MemoryRefStore` without navigating the internal
/// module hierarchy.
///
/// # Design rationale
/// Provides a clean and accessible API surface at the module root.
///
/// # Examples
///
/// ```
/// use libvctrl_core::store::MemoryRefStore;
/// use libvctrl_handler::{Hash, RefStore};
///
/// let mut store = MemoryRefStore::new();
/// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
/// store.set_ref("HEAD", &hash).unwrap();
/// assert!(store.get_ref("HEAD").is_ok());
/// ```
pub use ref_store::MemoryRefStore;
