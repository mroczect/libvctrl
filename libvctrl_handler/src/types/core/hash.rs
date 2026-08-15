use crate::constants::HASH_LENGTH;
use crate::errors::VctrlError;
use std::fmt;
use std::str::FromStr;

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
        let mut bytes = [0u8; HASH_LENGTH];
        for i in 0..HASH_LENGTH {
            let byte = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16)
                .map_err(|_| VctrlError::CorruptedData(format!("invalid hex char in hash: {s}")))?;
            bytes[i] = byte;
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
