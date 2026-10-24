use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use std::io::{self, Read, Write};

use libvctrl_handler::{Hash, Transport, TransportCapabilities, VctrlError};

#[derive(Debug)]
pub struct MemoryTransport {
    remote_refs: Vec<(String, Hash)>,

    remote_objects: Vec<(Hash, Vec<u8>)>,

    connected: bool,
}

impl Default for MemoryTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryTransport {
    pub fn new() -> Self {
        Self {
            remote_refs: Vec::new(),
            remote_objects: Vec::new(),
            connected: false,
        }
    }

    pub fn with_refs(refs: Vec<(String, Hash)>) -> Self {
        Self {
            remote_refs: refs,
            remote_objects: Vec::new(),
            connected: false,
        }
    }

    pub fn add_object(&mut self, hash: Hash, data: Vec<u8>) {
        self.remote_objects.push((hash, data));
    }
}

impl Transport for MemoryTransport {
    fn capabilities(&self) -> TransportCapabilities {
        TransportCapabilities::empty()
    }

    fn connect(&mut self, _url: &str) -> Result<(), VctrlError> {
        self.connected = true;
        Ok(())
    }

    fn disconnect(&mut self) -> Result<(), VctrlError> {
        self.connected = false;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn ls_refs(&self) -> Result<Vec<(String, Hash)>, VctrlError> {
        if !self.connected {
            return Err(VctrlError::Other("not connected".into()));
        }
        Ok(self.remote_refs.clone())
    }

    fn fetch_object(&self, hash: &Hash) -> Result<Vec<u8>, VctrlError> {
        if !self.connected {
            return Err(VctrlError::Other("not connected".into()));
        }

        self.remote_objects
            .iter()
            .find(|(h, _)| h == hash)
            .map(|(_, data)| data.clone())
            .ok_or_else(|| VctrlError::NotFound(hash.to_string()))
    }

    fn push_object(&mut self, hash: Hash, data: Vec<u8>) -> Result<(), VctrlError> {
        if !self.connected {
            return Err(VctrlError::Other("not connected".into()));
        }

        self.remote_objects.retain(|(h, _)| h != &hash);
        self.remote_objects.push((hash, data));
        Ok(())
    }

    fn update_ref(
        &mut self,
        ref_name: &str,
        old_hash: Option<&Hash>,
        new_hash: &Hash,
    ) -> Result<(), VctrlError> {
        if !self.connected {
            return Err(VctrlError::Other("not connected".into()));
        }

        if let Some(old) = old_hash {
            let current = self
                .remote_refs
                .iter()
                .find(|(name, _)| name == ref_name)
                .map(|(_, hash)| hash);

            if current != Some(old) {
                return Err(VctrlError::Other(
                    "ref update rejected: old hash mismatch".into(),
                ));
            }
        }

        if let Some(entry) = self
            .remote_refs
            .iter_mut()
            .find(|(name, _)| name == ref_name)
        {
            entry.1 = *new_hash;
        } else {
            self.remote_refs.push((ref_name.to_string(), *new_hash));
        }

        Ok(())
    }

    fn negotiate(&mut self, _wants: &[Hash], _haves: &[Hash]) -> Result<Vec<Hash>, VctrlError> {
        Ok(self.remote_objects.iter().map(|(h, _)| *h).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hash_byte(byte: u8) -> Hash {
        Hash::from_bytes(&[byte; 64]).unwrap()
    }

    #[test]
    fn connect_disconnect() {
        let mut transport = MemoryTransport::new();
        assert!(!transport.is_connected());

        transport.connect("http://example.com").unwrap();
        assert!(transport.is_connected());

        transport.disconnect().unwrap();
        assert!(!transport.is_connected());
    }

    #[test]
    fn ls_refs() {
        let transport = MemoryTransport::with_refs(vec![
            ("refs/heads/main".to_string(), hash_byte(0x01)),
            ("refs/tags/v1.0".to_string(), hash_byte(0x02)),
        ]);

        transport.connect("url").unwrap();
        let refs = transport.ls_refs().unwrap();

        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0].0, "refs/heads/main");
    }

    #[test]
    fn fetch_object() {
        let mut transport = MemoryTransport::new();
        transport.add_object(hash_byte(0x01), b"blob content".to_vec());

        transport.connect("url").unwrap();
        let data = transport.fetch_object(&hash_byte(0x01)).unwrap();
        assert_eq!(data, b"blob content");
    }

    #[test]
    fn fetch_missing_object() {
        let transport = MemoryTransport::new();
        transport.connect("url").unwrap();

        assert!(transport.fetch_object(&hash_byte(0x99)).is_err());
    }

    #[test]
    fn push_object() {
        let mut transport = MemoryTransport::new();
        transport.connect("url").unwrap();

        transport
            .push_object(hash_byte(0x01), b"new object".to_vec())
            .unwrap();
        let data = transport.fetch_object(&hash_byte(0x01)).unwrap();
        assert_eq!(data, b"new object");
    }

    #[test]
    fn update_ref() {
        let mut transport =
            MemoryTransport::with_refs(vec![("refs/heads/main".to_string(), hash_byte(0x01))]);

        transport.connect("url").unwrap();
        transport
            .update_ref("refs/heads/main", Some(&hash_byte(0x01)), &hash_byte(0x02))
            .unwrap();

        let refs = transport.ls_refs().unwrap();
        assert_eq!(refs[0].1, hash_byte(0x02));
    }

    #[test]
    fn update_ref_old_hash_mismatch() {
        let mut transport =
            MemoryTransport::with_refs(vec![("refs/heads/main".to_string(), hash_byte(0x01))]);

        transport.connect("url").unwrap();
        let result =
            transport.update_ref("refs/heads/main", Some(&hash_byte(0x99)), &hash_byte(0x02));
        assert!(result.is_err());
    }

    #[test]
    fn operations_fail_when_not_connected() {
        let transport = MemoryTransport::new();
        assert!(transport.ls_refs().is_err());
        assert!(transport.fetch_object(&hash_byte(0x01)).is_err());
    }
}
