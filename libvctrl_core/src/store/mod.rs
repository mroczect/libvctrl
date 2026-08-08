//! In‑memory reference implementations of the storage traits.
//!
//! This module provides **minimal, in‑memory** backends for the two core
//! storage abstractions defined in `libvctrl_handler`:
//!
//! | Struct | Trait | Backed by |
//! |---|---|---|
//! | [`MemoryStore`] | [`ObjectStore`] | [`std::collections::HashMap`] |
//! | [`MemoryRefStore`] | [`RefStore`] | [`std::collections::HashMap`] |
//!
//! # Purpose
//!
//! These stores are **not intended for production use**. They exist to:
//!
//! - Prove that the trait contracts can be implemented.
//! - Serve as a **reference** for building real backends (file‑system,
//!   database, cloud storage).
//! - Be used in **tests** and **prototypes** where persistence and
//!   concurrency are not required.
//!
//! # Limitations
//!
//! - **Not thread‑safe** – stores use plain `HashMap` without any
//!   synchronisation. To share them across threads, wrap them in an
//!   `Arc<Mutex<…>>`.
//! - **Not persistent** – all data is lost when the store is dropped.
//! - **No size limits** – stores will grow without bound. In production,
//!   you would add eviction policies or disk spilling.
//! - **No checksum verification** – objects are stored as raw bytes,
//!   exactly as provided. The store never verifies that a hash matches
//!   its content.
//!
//! # When to use
//!
//! Use these stores when you are:
//! - Writing unit tests for plumbing or porcelain functions.
//! - Prototyping a new feature before committing to a persistent backend.
//! - Learning how to implement the traits by studying a working example.
//!
//! For anything beyond that, build your own implementation or use a
//! community‑provided backend.
//!
//! # Examples
//!
//! ```rust
//! use libvctrl_core::store::{MemoryStore, MemoryRefStore};
//! use libvctrl_handler::{ObjectStore, RefStore, Hash, HASH_LENGTH};
//!
//! let hash = Hash::from_bytes(&[0xAB; HASH_LENGTH]).unwrap();
//!
//! // Object store
//! let mut obj_store = MemoryStore::new();
//! obj_store.put(&hash, b"some data").unwrap();
//! assert!(obj_store.exists(&hash).unwrap());
//! assert_eq!(obj_store.get(&hash).unwrap(), b"some data");
//!
//! // Reference store
//! let mut ref_store = MemoryRefStore::new();
//! ref_store.set_ref("HEAD", &hash).unwrap();
//! assert_eq!(ref_store.get_ref("HEAD").unwrap(), hash);
//! ```

pub mod memory;
pub mod ref_store;

pub use memory::MemoryStore;
pub use ref_store::MemoryRefStore;
