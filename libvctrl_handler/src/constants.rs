pub const HASH_LENGTH: usize = 64;

pub const MAX_NAME_LENGTH: usize = 255;

pub const MAX_BLOB_SIZE: usize = 100 * 1024 * 1024;

pub const MAX_TREE_ENTRIES: usize = 100_000;

pub const MAX_MESSAGE_LENGTH: usize = 1024 * 1024;

pub mod entry_mode {
    pub const BLOB: u32 = 0o100_644;

    pub const EXECUTABLE: u32 = 0o100_755;

    pub const SYMLINK: u32 = 0o120_000;

    pub const TREE: u32 = 0o040_000;

    pub const SUBMODULE: u32 = 0o160_000;
}
