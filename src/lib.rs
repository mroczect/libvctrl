pub mod core;
pub mod handler;

pub use core::backend::memory::{MemoryRefStore, MemoryStore};
pub use handler::{
    Blob, Commit, EntryKind, Hash, HashError, Object, ObjectStore, RefStore, Tree, TreeEntry,
    TreeError, UserInfo, VctrlError,
};
