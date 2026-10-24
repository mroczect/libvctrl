use alloc::string::String;
use alloc::vec::Vec;
use std::io;

use libvctrl_handler::{Hash, Signature, SignatureType, Signer, VctrlError, Verifier};

#[derive(Debug)]
pub struct MemorySigner {
    name: String,
    email: String,
    secret_key: Vec<u8>,
}

impl MemorySigner {
    pub fn new(name: impl Into<String>, email: impl Into<String>, secret_key: Vec<u8>) -> Self {
        Self {
            name: name.into(),
            email: email.into(),
            secret_key,
        }
    }
}

impl Signer for MemorySigner {
    fn sign(&self, data: &[u8], sig_type: SignatureType) -> Result<Signature, VctrlError> {
        let mut signature_bytes = Vec::with_capacity(data.len());
        for (i, &byte) in data.iter().enumerate() {
            let key_byte = self
                .secret_key
                .get(i % self.secret_key.len())
                .copied()
                .unwrap_or(0);
            signature_bytes.push(byte ^ key_byte);
        }

        Signature::new(
            self.name.clone(),
            self.email.clone(),
            sig_type,
            signature_bytes,
        )
    }

    fn sign_hash(&self, hash: &Hash, sig_type: SignatureType) -> Result<Signature, VctrlError> {
        self.sign(hash.as_bytes(), sig_type)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn email(&self) -> &str {
        &self.email
    }
}

#[derive(Debug)]
pub struct MemoryVerifier {
    public_keys: Vec<(String, Vec<u8>)>,
}

impl Default for MemoryVerifier {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryVerifier {
    pub fn new() -> Self {
        Self {
            public_keys: Vec::new(),
        }
    }

    pub fn add_public_key(&mut self, name: impl Into<String>, key: Vec<u8>) {
        self.public_keys.push((name.into(), key));
    }
}

impl Verifier for MemoryVerifier {
    fn verify(&self, data: &[u8], signature: &Signature) -> Result<bool, VctrlError> {
        let key = self
            .public_keys
            .iter()
            .find(|(name, _)| name == signature.signer_name())
            .map(|(_, key)| key.as_slice())
            .ok_or_else(|| {
                VctrlError::Other(format!("no public key for: {}", signature.signer_name()))
            })?;

        if signature.data().len() != data.len() {
            return Ok(false);
        }

        for (i, (&sig_byte, &data_byte)) in signature.data().iter().zip(data.iter()).enumerate() {
            let key_byte = key.get(i % key.len()).copied().unwrap_or(0);
            if (data_byte ^ key_byte) != sig_byte {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn verify_hash(&self, hash: &Hash, signature: &Signature) -> Result<bool, VctrlError> {
        self.verify(hash.as_bytes(), signature)
    }

    fn supported_types(&self) -> Vec<SignatureType> {
        vec![SignatureType::Unknown]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_and_verify_roundtrip() {
        let secret_key = b"test-secret-key".to_vec();
        let signer = MemorySigner::new("Alice", "alice@example.com", secret_key.clone());

        let data = b"important commit message";
        let signature = signer.sign(data, SignatureType::Unknown).unwrap();

        assert_eq!(signature.signer_name(), "Alice");
        assert_eq!(signature.signer_email(), "alice@example.com");

        let mut verifier = MemoryVerifier::new();
        verifier.add_public_key("Alice", secret_key);

        let valid = verifier.verify(data, &signature).unwrap();
        assert!(valid);
    }

    #[test]
    fn verify_wrong_data_fails() {
        let secret_key = b"secret".to_vec();
        let signer = MemorySigner::new("Bob", "bob@test.com", secret_key.clone());

        let signature = signer
            .sign(b"original data", SignatureType::Unknown)
            .unwrap();

        let mut verifier = MemoryVerifier::new();
        verifier.add_public_key("Bob", secret_key);

        let valid = verifier.verify(b"tampered data", &signature).unwrap();
        assert!(!valid);
    }

    #[test]
    fn verify_missing_key_errors() {
        let signer = MemorySigner::new("Alice", "a@b.com", vec![1, 2, 3]);
        let signature = signer.sign(b"data", SignatureType::Unknown).unwrap();

        let verifier = MemoryVerifier::new();
        assert!(verifier.verify(b"data", &signature).is_err());
    }

    #[test]
    fn sign_hash() {
        let signer = MemorySigner::new("Alice", "a@b.com", vec![0x42; 32]);
        let hash = Hash::from_bytes(&[0x01; 64]).unwrap();

        let sig = signer.sign_hash(&hash, SignatureType::Unknown).unwrap();
        assert_eq!(sig.signer_name(), "Alice");
    }
}
