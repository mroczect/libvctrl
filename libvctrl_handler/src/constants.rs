pub mod entry_mode {

    pub const BLOB: u32 = 0o100_644;

    pub const EXECUTABLE: u32 = 0o100_755;

    pub const SYMLINK: u32 = 0o120_000;

    pub const TREE: u32 = 0o40_000;

    pub const SUBMODULE: u32 = 0o160_000;
}

pub const HASH_LENGTH: usize = 64;

pub const MAX_NAME_LENGTH: u64 = 255;

pub const MAX_BLOB_SIZE: u64 = 100 * 1024 * 1024;

pub const MAX_TREE_ENTRIES: u64 = 100_000;

pub const MAX_MESSAGE_LENGTH: u64 = 1024 * 1024;

pub const MAX_PARENT_COUNT: u64 = 0xFFFF;
