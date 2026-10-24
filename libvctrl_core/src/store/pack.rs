use alloc::vec::Vec;
use std::io::{self, Cursor, Read, Write};

use libvctrl_handler::{
    EntryKind, HASH_LENGTH, Hash, PackEntry, PackObjectType, PackReader, PackWriter, VctrlError,
};

const PACK_SIGNATURE: &[u8; 4] = b"PACK";
const PACK_VERSION: u32 = 2;

#[derive(Debug)]
pub struct MemoryPackReader {
    entries: Vec<PackEntry>,
    index: BTreeMap<[u8; HASH_LENGTH], usize>,
    offset: usize,
}

use alloc::collections::BTreeMap;

impl Default for MemoryPackReader {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryPackReader {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            index: BTreeMap::new(),
            offset: 0,
        }
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, VctrlError> {
        let mut reader = Cursor::new(data);

        let mut sig = [0u8; 4];
        reader.read_exact(&mut sig).map_err(VctrlError::from_io)?;
        if &sig != PACK_SIGNATURE {
            return Err(VctrlError::CorruptedData("invalid pack signature".into()));
        }

        let mut version_bytes = [0u8; 4];
        reader
            .read_exact(&mut version_bytes)
            .map_err(VctrlError::from_io)?;
        let version = u32::from_be_bytes(version_bytes);
        if version != PACK_VERSION {
            return Err(VctrlError::CorruptedData(format!(
                "unsupported pack version: {version}"
            )));
        }

        let mut count_bytes = [0u8; 4];
        reader
            .read_exact(&mut count_bytes)
            .map_err(VctrlError::from_io)?;
        let _count = u32::from_be_bytes(count_bytes);

        Ok(Self::new())
    }

    pub fn add_entry(&mut self, entry: PackEntry) {
        let hash = *entry.hash().as_bytes();
        let pos = self.entries.len();
        self.index.insert(hash, pos);
        self.entries.push(entry);
    }
}

impl PackReader for MemoryPackReader {
    fn has_object(&self, hash: &Hash) -> bool {
        self.index.contains_key(hash.as_bytes())
    }

    fn read_header<R: Read + Send>(&mut self, _reader: &mut R) -> Result<PackEntry, VctrlError> {
        if self.offset >= self.entries.len() {
            return Err(VctrlError::IoError(alloc::sync::Arc::new(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "no more entries in pack",
            ))));
        }

        let entry = self.entries[self.offset].clone();
        self.offset += 1;
        Ok(entry)
    }

    fn read_object<R: Read + Send>(
        &mut self,
        _reader: &mut R,
        hash: &Hash,
    ) -> Result<Vec<u8>, VctrlError> {
        let pos = self
            .index
            .get(hash.as_bytes())
            .ok_or_else(|| VctrlError::NotFound(hash.to_string()))?;

        let entry = &self.entries[*pos];
        Ok(entry.data().to_vec())
    }

    fn entry_count(&self) -> usize {
        self.entries.len()
    }

    fn reset(&mut self) {
        self.offset = 0;
    }
}

#[derive(Debug, Default)]
pub struct MemoryPackWriter {
    entries: Vec<PackEntry>,
}

impl MemoryPackWriter {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn entries(&self) -> &[PackEntry] {
        &self.entries
    }
}

impl PackWriter for MemoryPackWriter {
    fn add_object(
        &mut self,
        obj_type: PackObjectType,
        hash: Hash,
        data: Vec<u8>,
    ) -> Result<(), VctrlError> {
        let entry = PackEntry::new(obj_type, hash, data)?;
        self.entries.push(entry);
        Ok(())
    }

    fn add_commit(&mut self, hash: Hash, data: Vec<u8>) -> Result<(), VctrlError> {
        self.add_object(PackObjectType::Commit, hash, data)
    }

    fn add_tree(&mut self, hash: Hash, data: Vec<u8>) -> Result<(), VctrlError> {
        self.add_object(PackObjectType::Tree, hash, data)
    }

    fn add_blob(&mut self, hash: Hash, data: Vec<u8>) -> Result<(), VctrlError> {
        self.add_object(PackObjectType::Blob, hash, data)
    }

    fn add_tag(&mut self, hash: Hash, data: Vec<u8>) -> Result<(), VctrlError> {
        self.add_object(PackObjectType::Tag, hash, data)
    }

    fn write_to<W: Write + Send>(&self, mut writer: W) -> Result<(), VctrlError> {
        writer
            .write_all(PACK_SIGNATURE)
            .map_err(VctrlError::from_io)?;
        writer
            .write_all(&PACK_VERSION.to_be_bytes())
            .map_err(VctrlError::from_io)?;
        writer
            .write_all(&(self.entries.len() as u32).to_be_bytes())
            .map_err(VctrlError::from_io)?;

        for entry in &self.entries {
            let type_byte = match entry.obj_type() {
                PackObjectType::Commit => 1,
                PackObjectType::Tree => 2,
                PackObjectType::Blob => 3,
                PackObjectType::Tag => 4,
            };

            let size = entry.data().len();
            let mut first_byte = (type_byte << 4) | (size as u8 & 0x0F);
            let mut shift = 4;

            if size > 0x0F {
                first_byte |= 0x80;
                writer
                    .write_all(&[first_byte])
                    .map_err(VctrlError::from_io)?;

                let mut remaining = size >> 4;
                while remaining >= 0x80 {
                    writer
                        .write_all(&[(remaining as u8) | 0x80])
                        .map_err(VctrlError::from_io)?;
                    remaining >>= 7;
                }
                writer
                    .write_all(&[remaining as u8])
                    .map_err(VctrlError::from_io)?;
            } else {
                writer
                    .write_all(&[first_byte])
                    .map_err(VctrlError::from_io)?;
            }

            writer
                .write_all(entry.data())
                .map_err(VctrlError::from_io)?;
        }

        writer.write_all(&[0u8; 20]).map_err(VctrlError::from_io)?;

        Ok(())
    }

    fn entry_count(&self) -> usize {
        self.entries.len()
    }

    fn clear(&mut self) {
        self.entries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hash_byte(byte: u8) -> Hash {
        Hash::from_bytes(&[byte; 64]).unwrap()
    }

    #[test]
    fn write_and_read_roundtrip() {
        let mut writer = MemoryPackWriter::new();

        let hash1 = hash_byte(0x01);
        let hash2 = hash_byte(0x02);

        writer.add_blob(hash1, b"hello".to_vec()).unwrap();
        writer.add_commit(hash2, b"commit data".to_vec()).unwrap();

        assert_eq!(writer.entry_count(), 2);

        let mut buf = Vec::new();
        writer.write_to(&mut buf).unwrap();

        assert_eq!(&buf[0..4], b"PACK");
        assert_eq!(u32::from_be_bytes(buf[4..8].try_into().unwrap()), 2);
    }

    #[test]
    fn reader_has_object() {
        let mut reader = MemoryPackReader::new();

        let hash = hash_byte(0x01);
        let entry = PackEntry::new(PackObjectType::Blob, hash, b"test".to_vec()).unwrap();

        reader.add_entry(entry);

        assert!(reader.has_object(&hash));
        assert!(!reader.has_object(&hash_byte(0x99)));
    }

    #[test]
    fn reader_read_header_sequential() {
        let mut reader = MemoryPackReader::new();

        reader.add_entry(PackEntry::new(PackObjectType::Blob, hash_byte(0x01), vec![1]).unwrap());
        reader.add_entry(PackEntry::new(PackObjectType::Tree, hash_byte(0x02), vec![2]).unwrap());

        let empty = &mut Cursor::new(Vec::<u8>::new());

        let h1 = reader.read_header(empty).unwrap();
        assert_eq!(*h1.hash(), hash_byte(0x01));

        let h2 = reader.read_header(empty).unwrap();
        assert_eq!(*h2.hash(), hash_byte(0x02));

        assert!(reader.read_header(empty).is_err());
    }

    #[test]
    fn reader_reset() {
        let mut reader = MemoryPackReader::new();
        reader.add_entry(PackEntry::new(PackObjectType::Blob, hash_byte(0x01), vec![]).unwrap());

        let empty = &mut Cursor::new(Vec::<u8>::new());
        reader.read_header(empty).unwrap();
        assert!(reader.read_header(empty).is_err());

        reader.reset();
        assert!(reader.read_header(empty).is_ok());
    }

    #[test]
    fn writer_clear() {
        let mut writer = MemoryPackWriter::new();
        writer.add_blob(hash_byte(0x01), vec![1]).unwrap();
        assert_eq!(writer.entry_count(), 1);

        writer.clear();
        assert_eq!(writer.entry_count(), 0);
    }
}
