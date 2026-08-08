//!   pure‑Rust, `#![no_std]` implementation that has been audited. It adds
use libvctrl_handler::{Hash, Hasher};
use libvctrl_sha512::Hash as Sha512Hash;

#[derive(Debug, Default, Clone)]
pub struct Sha512Hasher;

impl Hasher for Sha512Hasher {
    fn hash(&self, data: &[u8]) -> Hash {
        let digest = Sha512Hash::hash(data);
        Hash::from_bytes(&digest).expect("SHA-512 produces 64 bytes")
    }
}
