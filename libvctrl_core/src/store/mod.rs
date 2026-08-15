//! Store module.

/// In-memory object store.
pub mod memory;

/// In-memory reference store.
pub mod ref_store;

pub use memory::MemoryStore;
pub use ref_store::MemoryRefStore;
