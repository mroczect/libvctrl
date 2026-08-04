#![allow(dead_code)]

use libvctrl::{MemoryRefStore, MemoryStore, UserInfo};

pub fn setup_store() -> MemoryStore {
    MemoryStore::new()
}

pub fn setup_refs() -> MemoryRefStore {
    MemoryRefStore::new()
}

pub fn user(name: &str, email: &str) -> UserInfo {
    UserInfo::new(name.to_string(), email.to_string())
}

pub fn alice() -> UserInfo {
    user("Alice", "alice@example.com")
}

pub fn bob() -> UserInfo {
    user("Bob", "bob@example.com")
}
