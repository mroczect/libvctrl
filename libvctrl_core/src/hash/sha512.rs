//! SHA‑512 hasher.

use libvctrl_handler::{Hash, Hasher};
use libvctrl_sha512::Hash as Sha512Hash;

/// A [`Hasher`] implementation using SHA‑512.
///
/// Produces a 64‑byte digest, matching [`HASH_LENGTH`](libvctrl_handler::HASH_LENGTH).
///
/// # Examples
/// ```
/// use libvctrl_core::hash::Sha512Hasher;
/// use libvctrl_handler::Hasher;
///
/// let hasher = Sha512Hasher;
/// let hash = hasher.hash(b"hello");
/// assert_eq!(hash.as_bytes().len(), 64);
/// ```
#[derive(Debug, Default, Clone)]
pub struct Sha512Hasher;

impl Hasher for Sha512Hasher {
    fn hash(&self, data: &[u8]) -> Hash {
        let digest = Sha512Hash::hash(data);
        Hash::from_bytes(&digest).expect("SHA-512 produces 64 bytes")
    }
}
