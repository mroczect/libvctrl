use core::fmt;
use core::str::FromStr;

use crate::constants::HASH_LENGTH;
use crate::errors::VctrlError;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Hash([u8; HASH_LENGTH]);

impl Hash {
    #[allow(clippy::indexing_slicing)]
    pub const fn from_bytes(bytes: &[u8]) -> Result<Self, VctrlError> {
        if bytes.len() != HASH_LENGTH {
            return Err(VctrlError::InvalidHashLength(bytes.len()));
        }
        let mut arr = [0_u8; HASH_LENGTH];
        let mut i = 0;
        while i < HASH_LENGTH {
            arr[i] = bytes[i];
            i = i.wrapping_add(1);
        }
        Ok(Self(arr))
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; HASH_LENGTH] {
        &self.0
    }
}

impl From<[u8; HASH_LENGTH]> for Hash {
    fn from(arr: [u8; HASH_LENGTH]) -> Self {
        Self(arr)
    }
}

impl TryFrom<&[u8]> for Hash {
    type Error = VctrlError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Self::from_bytes(value)
    }
}

impl AsRef<[u8]> for Hash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl FromStr for Hash {
    type Err = VctrlError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != HASH_LENGTH * 2 {
            return Err(VctrlError::InvalidHashLength(s.len()));
        }
        let mut bytes = [0_u8; HASH_LENGTH];
        for (out, chunk) in bytes.iter_mut().zip(s.as_bytes().chunks_exact(2)) {
            let hex_str = core::str::from_utf8(chunk).map_err(|e| {
                VctrlError::CorruptedData(format!("invalid hex char in hash: {s}: {e}"))
            })?;
            *out = u8::from_str_radix(hex_str, 16).map_err(|e| {
                VctrlError::CorruptedData(format!("invalid hex char in hash: {s}: {e}"))
            })?;
        }
        Ok(Self(bytes))
    }
}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hash(")?;
        for &byte in self.0.iter().take(16) {
            write!(f, "{byte:02x}")?;
        }
        write!(f, "...)")
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
