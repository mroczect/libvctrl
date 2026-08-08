//! In‑memory reference implementations of the storage traits.
//!
//! These stores are not thread‑safe and not persistent.
//! They are intended for testing, prototyping, and as a reference
//! for building real backends.

pub mod memory;
pub mod ref_store;

pub use memory::MemoryStore;
pub use ref_store::MemoryRefStore;
