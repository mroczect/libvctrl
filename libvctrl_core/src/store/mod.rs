//! # In-Memory Stores
//!
//! This module provides ephemeral, in-memory implementations of the core
//! storage contracts defined in `libvctrl_handler`:
//!
//! - [`MemoryStore`] implements [`ObjectStore`](libvctrl_handler::ObjectStore)
//!   for storing and retrieving raw object bytes.
//! - [`MemoryRefStore`] implements [`RefStore`](libvctrl_handler::RefStore)
//!   for managing named references such as branches and tags.
//!
//! ## Why this module exists
//!
//! Version control backends must persist objects and references. However,
//! persistent storage requires platform-specific I/O and error handling. The
//! in-memory implementations decouple core VCS logic from those concerns.
//! They serve as:
//!
//! - Reference implementations for the traits.
//! - Test doubles for unit and integration tests.
//! - Backends for short-lived or embedded scenarios.
//!
//! ## How it works
//!
//! Both stores use [`std::collections::HashMap`] under the hood.
//!
//! - [`MemoryStore`] maps a [`Hash`] to raw encoded bytes (`Vec<u8>`).
//! - [`MemoryRefStore`] maps a reference name (`String`) to a [`Hash`].
//!
//! Lookups are O(1) on average. The reference store sorts names before
//! returning them from [`list_refs`](libvctrl_handler::RefStore::list_refs) to
//! provide deterministic iteration.
//!
//! ## Examples
//!
//! The following example shows how the two stores can be used together: an
//! object is placed into [`MemoryStore`], and a reference pointing to it is
//! stored in [`MemoryRefStore`].
//!
//! ```
//! # use libvctrl_handler::{Hash, ObjectStore, RefStore};
//! # use libvctrl_core::store::{MemoryStore, MemoryRefStore};
//! let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
//!
//! let mut object_store = MemoryStore::new();
//! object_store.put(&hash, b"encoded object bytes").unwrap();
//!
//! let mut ref_store = MemoryRefStore::new();
//! ref_store.set_ref("refs/heads/main", &hash).unwrap();
//!
//! assert!(object_store.exists(&hash).unwrap());
//! assert_eq!(ref_store.get_ref("refs/heads/main").unwrap(), hash);
//! ```

/// In-memory object store.
///
/// This submodule contains [`MemoryStore`](self::MemoryStore), a
/// [`HashMap`]-backed implementation of
/// [`ObjectStore`](libvctrl_handler::ObjectStore). It stores raw object bytes
/// and is suitable for testing and ephemeral storage.
pub mod memory;

/// In-memory reference store.
///
/// This submodule contains [`MemoryRefStore`](self::MemoryRefStore), a
/// [`HashMap`]-backed implementation of
/// [`RefStore`](libvctrl_handler::RefStore). It manages named references and
/// returns sorted reference names.
pub mod ref_store;

pub use memory::MemoryStore;
pub use ref_store::MemoryRefStore;
