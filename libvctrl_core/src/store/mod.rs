//! Ephemeral in-memory storage backends for `libvctrl_core`.
//!
//! # Purpose
//!
//! This module provides concrete, RAM-resident implementations of the
//! [`ObjectStore`](libvctrl_handler::ObjectStore) and
//! [`RefStore`](libvctrl_handler::RefStore) traits. These backends are
//! designed for testing, caching, and short-lived sessions where persistence
//! to disk or network is unnecessary.
//!
//! # Design Rationale
//!
//! - **Ephemeral state**: Data stored in these backends is lost when the
//!   process exits. This makes them ideal for unit tests where isolation and
//!   speed are critical. Tests can create a fresh store for each case,
//!   guaranteeing no cross-test contamination.
//! - **Structural separation**: Objects (content-addressed) and references
//!   (name-addressed) are kept in distinct stores. This mirrors the design of
//!   persistent version control systems (like Git) where loose objects and
//!   reference files occupy different directory structures. The separation
//!   allows each store to be optimized independently.
//! - **HashMap utilization**: Both stores leverage
//!   [`std::collections::HashMap`] to achieve average O(1) time complexity
//!   for insertions, lookups, and deletions. This makes in-memory operations
//!   extremely fast and predictable.
//!
//! # Internal Mechanism
//!
//! The [`MemoryStore`] maps a 64-byte [`Hash`](libvctrl_handler::Hash) to a
//! [`Vec<u8>`] payload, wrapping it in a [`std::io::Cursor`] for streaming
//! reads. The [`MemoryRefStore`] maps a [`String`] name to a
//! [`Hash`](libvctrl_handler::Hash). Both structs encapsulate their internal
//! maps as private fields, ensuring that all mutations occur through the
//! trait methods to enforce validation rules such as name length checks.
//!
//! # Complexities
//!
//! - `put` / `set_ref`: average O(1) insertion.
//! - `get` / `get_ref`: average O(1) lookup, with `get` also cloning the
//!   payload (O(n) where n is the object size).
//! - `delete` / `delete_ref`: average O(1) removal.
//! - `exists`: average O(1) key check.
//! - `list_refs`: O(n log n) due to sorting, where n is the number of
//!   references.
//!
//! # Thread Safety
//!
//! The in-memory stores are not [`Sync`] because [`HashMap`] is not safe for
//! concurrent access. If shared access is needed, wrap the store in a
//! [`Mutex`](std::sync::Mutex) or [`RwLock`](std::sync::RwLock). This
//! limitation is intentional; production backends with stronger concurrency
//! guarantees should implement the traits directly.
//!
//! # Examples
//!
//! Using the in-memory object store:
//!
//! ```
//! use libvctrl_core::store::MemoryStore;
//! use libvctrl_handler::{Hash, ObjectStore};
//! use std::io::Read;
//!
//! let mut store = MemoryStore::new();
//! let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
//! store.put(&hash, b"data").unwrap();
//!
//! let mut reader = store.get(&hash).unwrap();
//! let mut buf = Vec::new();
//! reader.read_to_end(&mut buf).unwrap();
//! assert_eq!(buf, b"data");
//! ```
//!
//! Using the in-memory reference store:
//!
//! ```
//! use libvctrl_core::store::MemoryRefStore;
//! use libvctrl_handler::{Hash, RefStore};
//!
//! let mut refs = MemoryRefStore::new();
//! let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
//! refs.set_ref("HEAD", &hash).unwrap();
//! assert_eq!(refs.get_ref("HEAD").unwrap(), hash);
//! ```

/// Module containing the [`MemoryStore`](crate::store::MemoryStore)
/// implementation.
///
/// # Purpose
///
/// Provides an in-memory key-value store for raw version control objects,
/// addressable by their cryptographic hash.
///
/// # Design Rationale
///
/// Encapsulating this logic in its own module isolates the storage mechanics
/// from the trait definitions, making the codebase easier to maintain and
/// test. The store uses a [`HashMap`] keyed by
/// [`Hash`](libvctrl_handler::Hash) for fast lookups.
///
/// # Streaming Reads
///
/// The `get` method returns a boxed reader backed by a cloned buffer. This
/// provides a safe, independent snapshot of the data while avoiding borrow
/// checker complications.
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

/// Module containing the [`MemoryRefStore`](crate::store::MemoryRefStore)
/// implementation.
///
/// # Purpose
///
/// Provides an in-memory store for named references (e.g., branches, tags)
/// that point to specific object hashes.
///
/// # Design Rationale
///
/// Separating reference storage from object storage allows reference updates
/// to be highly mutable without affecting the immutable object database. The
/// store enforces name validation to maintain compatibility with filesystem
/// backends and prevent resource exhaustion.
///
/// # Deterministic Listing
///
/// The `list_refs` method sorts reference names before returning them. This
/// guarantees reproducible output, which is crucial for testing and stable
/// user-facing listings.
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

/// Re-export of the [`MemoryStore`](crate::store::memory::MemoryStore)
/// struct.
///
/// # Purpose
///
/// Flattens the module path so users can simply import
/// `libvctrl_core::store::MemoryStore` instead of the full internal path
/// `libvctrl_core::store::memory::MemoryStore`.
///
/// # Design Rationale
///
/// Re-exporting at the module root reduces boilerplate and improves the
/// ergonomic experience for consumers of the crate. It also keeps the public
/// API stable even if the internal module layout changes.
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

/// Re-export of the [`MemoryRefStore`](crate::store::ref_store::MemoryRefStore)
/// struct.
///
/// # Purpose
///
/// Flattens the module path so users can simply import
/// `libvctrl_core::store::MemoryRefStore` without navigating the internal
/// module hierarchy.
///
/// # Design Rationale
///
/// Provides a clean and accessible API surface at the module root,
/// consistent with the rest of the crate.
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
