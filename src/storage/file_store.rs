use crate::codec::{BinaryDecoder, BinaryEncoder, Decoder, Encoder};
use crate::domain::Blob;
use crate::domain::hash::Hash;
use crate::domain::object::Object;
use crate::error::VctrlError;
use crate::storage::traits::{ObjectStore, RefStore};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::collections::{HashMap, HashSet};
use std::fs::{File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

const MAGIC: &[u8; 4] = b"VCTL";
const VERSION: u16 = 1;

const REC_BLOB: u8 = 0x01;
const REC_TREE: u8 = 0x02;
const REC_COMMIT: u8 = 0x03;
const REC_TAG: u8 = 0x04;
const REC_SET_REF: u8 = 0x10;
const REC_DEL_REF: u8 = 0x11;
const REC_SET_HEAD: u8 = 0x12;
const REC_DEL_OBJECT: u8 = 0x20;

struct ObjectInfo {
    rec_type: u8,
    offset: u64,
    length: u32,
}

pub struct FileStore {
    path: PathBuf,
    objects: HashMap<Hash, ObjectInfo>,
    refs: HashMap<String, Hash>,
    head: Option<String>,
    deleted: HashSet<Hash>,
    encoder: BinaryEncoder,
    decoder: BinaryDecoder,
    writer: Option<BufWriter<File>>,
}

impl FileStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, VctrlError> {
        let path = path.as_ref().to_path_buf();
        let mut store = Self {
            path: path.clone(),
            objects: HashMap::new(),
            refs: HashMap::new(),
            head: None,
            deleted: HashSet::new(),
            encoder: BinaryEncoder,
            decoder: BinaryDecoder,
            writer: None,
        };
        if path.exists() {
            store.load()?;
        } else {
            let mut file = BufWriter::new(
                OpenOptions::new()
                    .create_new(true)
                    .write(true)
                    .read(true)
                    .open(&path)
                    .map_err(VctrlError::Io)?,
            );
            file.write_all(MAGIC).map_err(VctrlError::Io)?;
            file.write_u16::<BigEndian>(VERSION)
                .map_err(VctrlError::Io)?;
            file.flush().map_err(VctrlError::Io)?;
            store.writer = Some(file);
        }
        Ok(store)
    }

    fn ensure_writer(&mut self) -> Result<&mut BufWriter<File>, VctrlError> {
        if self.writer.is_none() {
            let file = OpenOptions::new()
                .append(true)
                .open(&self.path)
                .map_err(VctrlError::Io)?;
            self.writer = Some(BufWriter::new(file));
        }
        self.writer
            .as_mut()
            .ok_or_else(|| VctrlError::Backend("no writer".into()))
    }

    fn load(&mut self) -> Result<(), VctrlError> {
        let mut file = BufReader::new(File::open(&self.path).map_err(VctrlError::Io)?);
        let mut magic = [0u8; 4];
        if file.read_exact(&mut magic).is_err() {
            return Err(VctrlError::Other("invalid file: too short".into()));
        }
        if &magic != MAGIC {
            return Err(VctrlError::Other("invalid magic".into()));
        }
        let version = file.read_u16::<BigEndian>().map_err(VctrlError::Io)?;
        if version != VERSION {
            return Err(VctrlError::Other(format!(
                "unsupported version {}",
                version
            )));
        }
        let mut deleted_hashes = HashSet::new();
        loop {
            let rec_type = match file.read_u8() {
                Ok(t) => t,
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(VctrlError::Io(e)),
            };
            match rec_type {
                REC_BLOB | REC_TREE | REC_COMMIT | REC_TAG => {
                    let mut hash_bytes = [0u8; 64];
                    file.read_exact(&mut hash_bytes).map_err(VctrlError::Io)?;
                    let hash = Hash::from_bytes(hash_bytes);
                    let length = file.read_u32::<BigEndian>().map_err(VctrlError::Io)?;
                    let offset = file.stream_position().map_err(VctrlError::Io)?;
                    file.seek(SeekFrom::Current(length as i64))
                        .map_err(VctrlError::Io)?;
                    if !deleted_hashes.contains(&hash) {
                        self.objects.insert(
                            hash,
                            ObjectInfo {
                                rec_type,
                                offset,
                                length,
                            },
                        );
                    }
                }
                REC_SET_REF => {
                    let name_len = file.read_u16::<BigEndian>().map_err(VctrlError::Io)?;
                    let mut name = vec![0u8; name_len as usize];
                    file.read_exact(&mut name).map_err(VctrlError::Io)?;
                    let name =
                        String::from_utf8(name).map_err(|e| VctrlError::Other(e.to_string()))?;
                    let mut h = [0u8; 64];
                    file.read_exact(&mut h).map_err(VctrlError::Io)?;
                    self.refs.insert(name, Hash::from_bytes(h));
                }
                REC_DEL_REF => {
                    let name_len = file.read_u16::<BigEndian>().map_err(VctrlError::Io)?;
                    let mut name = vec![0u8; name_len as usize];
                    file.read_exact(&mut name).map_err(VctrlError::Io)?;
                    let name =
                        String::from_utf8(name).map_err(|e| VctrlError::Other(e.to_string()))?;
                    self.refs.remove(&name);
                }
                REC_SET_HEAD => {
                    let target_len = file.read_u16::<BigEndian>().map_err(VctrlError::Io)?;
                    let mut target = vec![0u8; target_len as usize];
                    file.read_exact(&mut target).map_err(VctrlError::Io)?;
                    let target =
                        String::from_utf8(target).map_err(|e| VctrlError::Other(e.to_string()))?;
                    self.head = Some(target);
                }
                REC_DEL_OBJECT => {
                    let mut h = [0u8; 64];
                    file.read_exact(&mut h).map_err(VctrlError::Io)?;
                    deleted_hashes.insert(Hash::from_bytes(h));
                }
                _ => {
                    return Err(VctrlError::Other(format!(
                        "unknown record type {}",
                        rec_type
                    )));
                }
            }
        }
        for hash in &deleted_hashes {
            self.objects.remove(hash);
        }
        self.deleted = deleted_hashes;
        Ok(())
    }

    fn encode_object(&self, obj: &Object) -> Result<(u8, Vec<u8>), VctrlError> {
        match obj {
            Object::Blob(blob) => Ok((REC_BLOB, blob.as_bytes().to_vec())),
            Object::Tree(tree) => {
                let mut buf = Vec::new();
                self.encoder.encode_tree(tree, &mut buf)?;
                Ok((REC_TREE, buf))
            }
            Object::Commit(commit) => {
                let mut buf = Vec::new();
                self.encoder.encode_commit(commit, &mut buf)?;
                Ok((REC_COMMIT, buf))
            }
            Object::Tag(tag) => {
                let mut buf = Vec::new();
                self.encoder.encode_tag(tag, &mut buf)?;
                Ok((REC_TAG, buf))
            }
        }
    }

    fn decode_object(&self, rec_type: u8, data: &[u8]) -> Result<Object, VctrlError> {
        match rec_type {
            REC_BLOB => Ok(Object::Blob(Blob::new(data.to_vec()))),
            REC_TREE => {
                let tree = self.decoder.decode_tree(data)?;
                Ok(Object::Tree(tree))
            }
            REC_COMMIT => {
                let commit = self.decoder.decode_commit(data)?;
                Ok(Object::Commit(Box::new(commit)))
            }
            REC_TAG => {
                let tag = self.decoder.decode_tag(data)?;
                Ok(Object::Tag(Box::new(tag)))
            }
            _ => Err(VctrlError::Other("invalid object record type".into())),
        }
    }
}

impl ObjectStore for FileStore {
    fn put(&mut self, hash: &Hash, obj: &Object) -> Result<(), VctrlError> {
        if self.objects.contains_key(hash) {
            return Ok(());
        }
        self.deleted.remove(hash);
        let (rec_type, data) = self.encode_object(obj)?;
        let writer = self.ensure_writer()?;
        writer.write_u8(rec_type).map_err(VctrlError::Io)?;
        writer.write_all(hash.as_bytes()).map_err(VctrlError::Io)?;
        writer
            .write_u32::<BigEndian>(data.len() as u32)
            .map_err(VctrlError::Io)?;
        let data_offset = writer.stream_position().map_err(VctrlError::Io)?;
        writer.write_all(&data).map_err(VctrlError::Io)?;
        writer.flush().map_err(VctrlError::Io)?;
        writer.get_mut().sync_all().map_err(VctrlError::Io)?;
        self.objects.insert(
            *hash,
            ObjectInfo {
                rec_type,
                offset: data_offset,
                length: data.len() as u32,
            },
        );
        Ok(())
    }

    fn get(&self, hash: &Hash) -> Result<Option<Object>, VctrlError> {
        if self.deleted.contains(hash) {
            return Ok(None);
        }
        match self.objects.get(hash) {
            Some(info) => {
                let mut file = File::open(&self.path).map_err(VctrlError::Io)?;
                file.seek(SeekFrom::Start(info.offset))
                    .map_err(VctrlError::Io)?;
                let mut buf = vec![0u8; info.length as usize];
                file.read_exact(&mut buf).map_err(VctrlError::Io)?;
                let obj = self.decode_object(info.rec_type, &buf)?;
                Ok(Some(obj))
            }
            None => Ok(None),
        }
    }

    fn exists(&self, hash: &Hash) -> Result<bool, VctrlError> {
        Ok(self.objects.contains_key(hash) && !self.deleted.contains(hash))
    }

    fn all_hashes(&self) -> Result<Vec<Hash>, VctrlError> {
        Ok(self
            .objects
            .keys()
            .filter(|h| !self.deleted.contains(h))
            .copied()
            .collect())
    }

    fn remove(&mut self, hash: &Hash) -> Result<(), VctrlError> {
        if self.objects.remove(hash).is_some() {
            self.deleted.insert(*hash);
            let writer = self.ensure_writer()?;
            writer.write_u8(REC_DEL_OBJECT).map_err(VctrlError::Io)?;
            writer.write_all(hash.as_bytes()).map_err(VctrlError::Io)?;
            writer.flush().map_err(VctrlError::Io)?;
            Ok(())
        } else {
            Err(VctrlError::NotFound(format!("object '{}' not found", hash)))
        }
    }
}

impl RefStore for FileStore {
    fn set_ref(&mut self, name: &str, hash: &Hash) -> Result<(), VctrlError> {
        let writer = self.ensure_writer()?;
        writer.write_u8(REC_SET_REF).map_err(VctrlError::Io)?;
        let name_bytes = name.as_bytes();
        writer
            .write_u16::<BigEndian>(name_bytes.len() as u16)
            .map_err(VctrlError::Io)?;
        writer.write_all(name_bytes).map_err(VctrlError::Io)?;
        writer.write_all(hash.as_bytes()).map_err(VctrlError::Io)?;
        writer.flush().map_err(VctrlError::Io)?;
        self.refs.insert(name.to_string(), *hash);
        Ok(())
    }

    fn get_ref(&self, name: &str) -> Result<Option<Hash>, VctrlError> {
        Ok(self.refs.get(name).copied())
    }

    fn delete_ref(&mut self, name: &str) -> Result<(), VctrlError> {
        if self.refs.remove(name).is_some() {
            let writer = self.ensure_writer()?;
            writer.write_u8(REC_DEL_REF).map_err(VctrlError::Io)?;
            let name_bytes = name.as_bytes();
            writer
                .write_u16::<BigEndian>(name_bytes.len() as u16)
                .map_err(VctrlError::Io)?;
            writer.write_all(name_bytes).map_err(VctrlError::Io)?;
            writer.flush().map_err(VctrlError::Io)?;
        }
        Ok(())
    }

    fn set_head(&mut self, target: &str) -> Result<(), VctrlError> {
        let writer = self.ensure_writer()?;
        writer.write_u8(REC_SET_HEAD).map_err(VctrlError::Io)?;
        let target_bytes = target.as_bytes();
        writer
            .write_u16::<BigEndian>(target_bytes.len() as u16)
            .map_err(VctrlError::Io)?;
        writer.write_all(target_bytes).map_err(VctrlError::Io)?;
        writer.flush().map_err(VctrlError::Io)?;
        self.head = Some(target.to_string());
        Ok(())
    }

    fn head(&self) -> Result<Option<Hash>, VctrlError> {
        match &self.head {
            Some(target) if target.starts_with("refs/") => self.get_ref(target),
            Some(direct) => Hash::from_hex(direct).map(Some).map_err(VctrlError::Hash),
            None => Ok(None),
        }
    }

    fn head_ref_name(&self) -> Result<Option<String>, VctrlError> {
        match &self.head {
            Some(target) if target.starts_with("refs/heads/") => {
                if self.refs.contains_key(target) {
                    Ok(Some(target.clone()))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    fn list_refs(&self, prefix: &str) -> Result<Vec<String>, VctrlError> {
        Ok(self
            .refs
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect())
    }
}
