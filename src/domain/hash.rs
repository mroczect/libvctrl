use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Hash([u8; 64]);

impl Hash {
    pub const fn from_bytes(bytes: [u8; 64]) -> Self {
        Self(bytes)
    }
    pub fn from_slice(slice: &[u8]) -> Result<Self, HashError> {
        let array: [u8; 64] = slice
            .try_into()
            .map_err(|_| HashError::InvalidLength(slice.len()))?;
        Ok(Self(array))
    }
    pub fn from_hex(hex: &str) -> Result<Self, HashError> {
        let bytes = hex::decode(hex).map_err(|_| HashError::InvalidHex)?;
        Self::from_slice(&bytes)
    }
    pub const fn as_bytes(&self) -> &[u8; 64] {
        &self.0
    }
    pub fn to_hex(&self) -> String {
        use std::fmt::Write;
        let mut s = String::with_capacity(128);
        for b in &self.0 {
            write!(s, "{:02x}", b).expect("write to String cannot fail");
        }
        s
    }
}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hash({})", self.to_hex())
    }
}
impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_hex())
    }
}
impl FromStr for Hash {
    type Err = HashError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_hex(s)
    }
}
impl Serialize for Hash {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_hex())
    }
}
impl<'de> Deserialize<'de> for Hash {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Hash::from_hex(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum HashError {
    #[error("invalid hash length: {0} (expected 64)")]
    InvalidLength(usize),
    #[error("invalid hex string")]
    InvalidHex,
}

mod hex {
    pub fn decode(hex: &str) -> Result<Vec<u8>, ()> {
        if !hex.len().is_multiple_of(2) {
            return Err(());
        }
        (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).map_err(|_| ()))
            .collect()
    }
}
