//!         Hash::from_bytes(&digest).expect("must produce 64 bytes")
pub mod sha512;
pub use sha512::Sha512Hasher;
