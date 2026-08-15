use crate::constants::HASH_LENGTH;
use crate::errors::VctrlError;
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Hash([u8; HASH_LENGTH]);

impl Hash {
    pub const fn from_bytes(bytes: &[u8]) -> Result<Self, VctrlError> {
        if bytes.len() != HASH_LENGTH {
            return Err(VctrlError::InvalidHashLength(bytes.len()));
        }
        let mut arr = [0u8; HASH_LENGTH];
        let mut i = 0;
        while i < HASH_LENGTH {
            arr[i] = bytes[i];
            i += 1;
        }
        Ok(Self(arr))
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; HASH_LENGTH] {
        &self.0
    }
}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hash(")?;
        for &byte in self.0.iter().take(8) {
            write!(f, "{byte:02x}")?;
        }
        write!(f, "…)")
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &byte in &self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}
