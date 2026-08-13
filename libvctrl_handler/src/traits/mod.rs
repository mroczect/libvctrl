//! # Traits Module
//!
//! This module defines the abstract interfaces that concrete version control
//! backends must implement. By defining these as traits, the core data types
//! remain completely decoupled from storage, networking, and serialization
//! logic.
//!
//! ## Design Rationale
//!
//! The traits are split by responsibility:
//!
//! - **`ObjectStore`** and **`RefStore`** handle persistence.
//! - **`Encoder`** and **`Decoder`** handle serialization.
//! - **`Hasher`** handles content addressing.
//! - **`Signer`** and **`Verifier`** handle cryptographic integrity.
//! - **`Transport`** handles remote synchronization.
//!
//! This separation of concerns allows mixing and matching implementations
//! (e.g., an in-memory store with a binary encoder) and vastly simplifies
//! unit testing of individual components.
//!
//! ## Module Structure
//!
//! Each trait resides in its own file under the [`core`] submodule to
//! improve maintainability and reduce merge conflicts. The [`core`] module
//! contains the following submodules:
//!
//! - [`decoder`](core::decoder)
//! - [`encoder`](core::encoder)
//! - [`hasher`](core::hasher)
//! - [`object_store`](core::object_store)
//! - [`ref_store`](core::ref_store)
//! - [`signer`](core::signer)
//! - [`transport`](core::transport)
//! - [`verifier`](core::verifier)
//!
//! ## Streaming and Memory Efficiency
//!
//! Starting from version 3.2, [`ObjectStore::get`](crate::ObjectStore::get)
//! returns a [`Box<dyn std::io::Read>`] instead of a [`Vec<u8>`]. This
//! enables streaming of object data directly from the backing store without
//! forcing large contiguous allocations. The reader is borrowed from `&self`,
//! so the store cannot be mutated while a reader exists. This invariant is
//! enforced by Rust's borrow checker.
//!
//! ## How to Implement a Trait
//!
//! Each trait is re-exported at the crate root. To implement a trait, import
//! it from `libvctrl_handler` and provide the required methods. For example,
//! implementing [`Hasher`](crate::Hasher):
//!
//! ```
//! use libvctrl_handler::{Hash, Hasher, VctrlError};
//!
//! struct DummyHasher;
//!
//! impl Hasher for DummyHasher {
//!     fn hash(&self, _data: &[u8]) -> Result<Hash, VctrlError> {
//!         Ok(Hash::from_bytes(&[0u8; 64]).unwrap())
//!     }
//! }
//!
//! let hasher = DummyHasher;
//! let hash = hasher.hash(b"payload").unwrap();
//! assert_eq!(hash.as_bytes().len(), 64);
//! ```
//!
//! ## Implementing an Object Store
//!
//! A complete in-memory implementation of [`ObjectStore`](crate::ObjectStore)
//! demonstrates the streaming read API:
//!
//! ```
//! use libvctrl_handler::{Hash, ObjectStore, VctrlError};
//! use std::collections::HashMap;
//! use std::io::Read;
//!
//! #[derive(Default)]
//! struct InMemoryStore(HashMap<Hash, Vec<u8>>);
//!
//! impl ObjectStore for InMemoryStore {
//!     fn put(&mut self, hash: &Hash, data: &[u8]) -> Result<(), VctrlError> {
//!         self.0.insert(*hash, data.to_vec());
//!         Ok(())
//!     }
//!
//!     fn get(&self, hash: &Hash) -> Result<Box<dyn Read + '_>, VctrlError> {
//!         self.0
//!             .get(hash)
//!             .cloned()
//!             .map(|v| Box::new(std::io::Cursor::new(v)) as Box<dyn Read>)
//!             .ok_or_else(|| VctrlError::ObjectNotFound(*hash))
//!     }
//!
//!     fn delete(&mut self, hash: &Hash) -> Result<(), VctrlError> {
//!         self.0.remove(hash);
//!         Ok(())
//!     }
//!
//!     fn exists(&self, hash: &Hash) -> Result<bool, VctrlError> {
//!         Ok(self.0.contains_key(hash))
//!     }
//! }
//!
//! let mut store = InMemoryStore::default();
//! let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
//! store.put(&hash, b"data").unwrap();
//!
//! let mut reader = store.get(&hash).unwrap();
//! let mut buf = Vec::new();
//! reader.read_to_end(&mut buf).unwrap();
//! assert_eq!(buf, b"data");
//! ```
//!
//! ## Using Multiple Traits Together
//!
//! The trait separation encourages composition. A typical backend might use
//! one struct to implement both [`Encoder`](crate::Encoder) and
//! [`Hasher`](crate::Hasher), while another handles storage. This modularity
//! allows the same core data types to be used with different concrete
//! backends without modification.

/// Core behavior contracts (traits) for storage, encoding, hashing, and
/// transport.
///
/// # Purpose
///
/// This submodule contains the actual trait definitions. Each trait lives in
/// its own file under this module, keeping compilation units small and
/// dependencies explicit.
///
/// # Why a `core` submodule?
///
/// Grouping traits under `core` provides a clear internal structure while
/// allowing the parent `traits` module to re-export them at a higher level.
/// This pattern is repeated elsewhere in the crate (for example, in
/// [`types`](crate::types)).
///
/// # List of Traits
///
/// - [`Decoder`](crate::Decoder): deserializes objects from byte slices.
/// - [`Encoder`](crate::Encoder): serializes objects into byte vectors.
/// - [`Hasher`](crate::Hasher): computes cryptographic hashes.
/// - [`ObjectStore`](crate::ObjectStore): stores and retrieves raw objects.
/// - [`RefStore`](crate::RefStore): manages named references.
/// - [`Signer`](crate::Signer): signs data cryptographically.
/// - [`Transport`](crate::Transport): fetches and pushes objects remotely.
/// - [`Verifier`](crate::Verifier): verifies cryptographic signatures.
///
/// # Examples
///
/// Importing a trait from the `core` path:
///
/// ```
/// use libvctrl_handler::traits::core::hasher::Hasher;
///
/// // `Hasher` is also available at the crate root:
/// use libvctrl_handler::Hasher as RootHasher;
///
/// // Both refer to the same trait.
/// fn _assert_same_trait(_: impl Hasher) {}
/// ```
///
/// However, for ergonomic use, prefer the crate root re-exports:
///
/// ```
/// use libvctrl_handler::Hasher;
/// ```
///
/// # Design Note
///
/// The `core` module is public to allow advanced users to refer to traits
/// by their full path if needed, but the intended public API is through the
/// crate root.
pub mod core;
