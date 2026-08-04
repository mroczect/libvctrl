use crate::codec::{Decoder, Encoder};
use crate::domain::commit::Commit;
use crate::domain::hash::Hash;
use crate::domain::tag::Tag;
use crate::domain::tree::{EntryKind, Tree, TreeEntry};
use crate::domain::user::UserID;
use crate::error::VctrlError;
use byteorder::{BigEndian, ReadBytesExt};
use chrono::{TimeZone, Utc};
use std::io::{Cursor, Read};

pub struct BinaryEncoder;

impl Encoder for BinaryEncoder {
    fn encode_tree(&self, tree: &Tree, buf: &mut Vec<u8>) -> Result<(), VctrlError> {
        buf.push(1u8);
        let n = u32::try_from(tree.entries().len())
            .map_err(|_| VctrlError::Other("tree has too many entries to encode".into()))?;
        buf.extend_from_slice(&n.to_be_bytes());
        for entry in tree.entries() {
            let name = entry.name.as_bytes();
            let name_len = u16::try_from(name.len()).map_err(|_| {
                VctrlError::Other(format!("entry name '{}' too long to encode", entry.name))
            })?;
            buf.extend_from_slice(&name_len.to_be_bytes());
            buf.extend_from_slice(name);
            match entry.kind {
                EntryKind::Blob => buf.push(0u8),
                EntryKind::Tree => buf.push(1u8),
            }
            buf.extend_from_slice(entry.hash.as_bytes());
        }
        Ok(())
    }

    fn encode_commit(&self, commit: &Commit, buf: &mut Vec<u8>) -> Result<(), VctrlError> {
        buf.push(1u8);
        buf.extend_from_slice(commit.tree.as_bytes());
        let np = u32::try_from(commit.parents.len())
            .map_err(|_| VctrlError::Other("commit has too many parents".into()))?;
        buf.extend_from_slice(&np.to_be_bytes());
        for p in &commit.parents {
            buf.extend_from_slice(p.as_bytes());
        }
        write_user(&commit.author, buf)?;
        write_user(&commit.committer, buf)?;
        let ts = commit.timestamp.timestamp();
        let ts_ns = commit.timestamp.timestamp_subsec_nanos();
        buf.extend_from_slice(&ts.to_be_bytes());
        buf.extend_from_slice(&ts_ns.to_be_bytes());
        let msg = commit.message.as_bytes();
        let msg_len = u32::try_from(msg.len())
            .map_err(|_| VctrlError::Other("commit message too long".into()))?;
        buf.extend_from_slice(&msg_len.to_be_bytes());
        buf.extend_from_slice(msg);
        if let Some(sig) = &commit.signature {
            let sig_len = u32::try_from(sig.len())
                .map_err(|_| VctrlError::Other("signature too long".into()))?;
            buf.extend_from_slice(&sig_len.to_be_bytes());
            buf.extend_from_slice(sig);
        } else {
            buf.extend_from_slice(&0u32.to_be_bytes());
        }
        Ok(())
    }

    fn encode_tag(&self, tag: &Tag, buf: &mut Vec<u8>) -> Result<(), VctrlError> {
        buf.push(1u8);
        buf.extend_from_slice(tag.target.as_bytes());
        write_user(&tag.tagger, buf)?;
        let ts = tag.timestamp.timestamp();
        let ts_ns = tag.timestamp.timestamp_subsec_nanos();
        buf.extend_from_slice(&ts.to_be_bytes());
        buf.extend_from_slice(&ts_ns.to_be_bytes());
        let msg = tag.message.as_bytes();
        let msg_len = u32::try_from(msg.len())
            .map_err(|_| VctrlError::Other("tag message too long".into()))?;
        buf.extend_from_slice(&msg_len.to_be_bytes());
        buf.extend_from_slice(msg);
        Ok(())
    }
}

fn write_user(user: &UserID, buf: &mut Vec<u8>) -> Result<(), VctrlError> {
    let name = user.name.as_bytes();
    let name_len = u16::try_from(name.len())
        .map_err(|_| VctrlError::Other(format!("name '{}' too long", user.name)))?;
    buf.extend_from_slice(&name_len.to_be_bytes());
    buf.extend_from_slice(name);

    let email = user.email.as_bytes();
    let email_len = u16::try_from(email.len())
        .map_err(|_| VctrlError::Other(format!("email '{}' too long", user.email)))?;
    buf.extend_from_slice(&email_len.to_be_bytes());
    buf.extend_from_slice(email);
    Ok(())
}

pub struct BinaryDecoder;

impl Decoder for BinaryDecoder {
    fn decode_tree(&self, data: &[u8]) -> Result<Tree, VctrlError> {
        let mut cursor = Cursor::new(data);
        let version = cursor
            .read_u8()
            .map_err(|e| VctrlError::Other(e.to_string()))?;
        if version != 1 {
            return Err(VctrlError::Other("unsupported tree version".into()));
        }
        let n = cursor
            .read_u32::<BigEndian>()
            .map_err(|e| VctrlError::Other(e.to_string()))?;
        let mut entries = Vec::with_capacity(n as usize);
        for _ in 0..n {
            let name_len = cursor
                .read_u16::<BigEndian>()
                .map_err(|e| VctrlError::Other(e.to_string()))?;
            let mut name_bytes = vec![0u8; name_len as usize];
            cursor
                .read_exact(&mut name_bytes)
                .map_err(|e| VctrlError::Other(e.to_string()))?;
            let name =
                String::from_utf8(name_bytes).map_err(|e| VctrlError::Other(e.to_string()))?;
            let kind_byte = cursor
                .read_u8()
                .map_err(|e| VctrlError::Other(e.to_string()))?;
            let kind = match kind_byte {
                0 => EntryKind::Blob,
                1 => EntryKind::Tree,
                _ => return Err(VctrlError::Other("invalid entry kind".into())),
            };
            let mut hash_bytes = [0u8; 64];
            cursor
                .read_exact(&mut hash_bytes)
                .map_err(|e| VctrlError::Other(e.to_string()))?;
            let hash = Hash::from_bytes(hash_bytes);
            entries.push(TreeEntry { name, kind, hash });
        }
        Tree::new(entries).map_err(VctrlError::Tree)
    }

    fn decode_commit(&self, data: &[u8]) -> Result<Commit, VctrlError> {
        let mut cursor = Cursor::new(data);
        let version = cursor
            .read_u8()
            .map_err(|e| VctrlError::Other(e.to_string()))?;
        if version != 1 {
            return Err(VctrlError::Other("unsupported commit version".into()));
        }
        let mut tree_hash = [0u8; 64];
        cursor
            .read_exact(&mut tree_hash)
            .map_err(|e| VctrlError::Other(e.to_string()))?;
        let tree = Hash::from_bytes(tree_hash);
        let np = cursor
            .read_u32::<BigEndian>()
            .map_err(|e| VctrlError::Other(e.to_string()))?;
        let mut parents = Vec::with_capacity(np as usize);
        for _ in 0..np {
            let mut h = [0u8; 64];
            cursor
                .read_exact(&mut h)
                .map_err(|e| VctrlError::Other(e.to_string()))?;
            parents.push(Hash::from_bytes(h));
        }
        let author = read_user(&mut cursor)?;
        let committer = read_user(&mut cursor)?;
        let ts = cursor
            .read_i64::<BigEndian>()
            .map_err(|e| VctrlError::Other(e.to_string()))?;
        let ts_ns = cursor
            .read_u32::<BigEndian>()
            .map_err(|e| VctrlError::Other(e.to_string()))?;
        let timestamp = Utc
            .timestamp_opt(ts, ts_ns)
            .single()
            .ok_or_else(|| VctrlError::Other("invalid timestamp".into()))?;
        let msg_len = cursor
            .read_u32::<BigEndian>()
            .map_err(|e| VctrlError::Other(e.to_string()))?;
        let mut msg_bytes = vec![0u8; msg_len as usize];
        cursor
            .read_exact(&mut msg_bytes)
            .map_err(|e| VctrlError::Other(e.to_string()))?;
        let message = String::from_utf8(msg_bytes).map_err(|e| VctrlError::Other(e.to_string()))?;
        let sig_len = cursor
            .read_u32::<BigEndian>()
            .map_err(|e| VctrlError::Other(e.to_string()))?;
        let signature = if sig_len == 0 {
            None
        } else {
            let mut sig = vec![0u8; sig_len as usize];
            cursor
                .read_exact(&mut sig)
                .map_err(|e| VctrlError::Other(e.to_string()))?;
            Some(sig)
        };
        Ok(Commit {
            tree,
            parents,
            author,
            committer,
            timestamp,
            message,
            signature,
        })
    }

    fn decode_tag(&self, data: &[u8]) -> Result<Tag, VctrlError> {
        let mut cursor = Cursor::new(data);
        let version = cursor
            .read_u8()
            .map_err(|e| VctrlError::Other(e.to_string()))?;
        if version != 1 {
            return Err(VctrlError::Other("unsupported tag version".into()));
        }
        let mut target_hash = [0u8; 64];
        cursor
            .read_exact(&mut target_hash)
            .map_err(|e| VctrlError::Other(e.to_string()))?;
        let target = Hash::from_bytes(target_hash);
        let tagger = read_user(&mut cursor)?;
        let ts = cursor
            .read_i64::<BigEndian>()
            .map_err(|e| VctrlError::Other(e.to_string()))?;
        let ts_ns = cursor
            .read_u32::<BigEndian>()
            .map_err(|e| VctrlError::Other(e.to_string()))?;
        let timestamp = Utc
            .timestamp_opt(ts, ts_ns)
            .single()
            .ok_or_else(|| VctrlError::Other("invalid timestamp".into()))?;
        let msg_len = cursor
            .read_u32::<BigEndian>()
            .map_err(|e| VctrlError::Other(e.to_string()))?;
        let mut msg_bytes = vec![0u8; msg_len as usize];
        cursor
            .read_exact(&mut msg_bytes)
            .map_err(|e| VctrlError::Other(e.to_string()))?;
        let message = String::from_utf8(msg_bytes).map_err(|e| VctrlError::Other(e.to_string()))?;
        Ok(Tag {
            target,
            tagger,
            timestamp,
            message,
        })
    }
}

fn read_user(cursor: &mut Cursor<&[u8]>) -> Result<UserID, VctrlError> {
    let name_len = cursor
        .read_u16::<BigEndian>()
        .map_err(|e| VctrlError::Other(e.to_string()))?;
    let mut name_bytes = vec![0u8; name_len as usize];
    cursor
        .read_exact(&mut name_bytes)
        .map_err(|e| VctrlError::Other(e.to_string()))?;
    let name = String::from_utf8(name_bytes).map_err(|e| VctrlError::Other(e.to_string()))?;
    let email_len = cursor
        .read_u16::<BigEndian>()
        .map_err(|e| VctrlError::Other(e.to_string()))?;
    let mut email_bytes = vec![0u8; email_len as usize];
    cursor
        .read_exact(&mut email_bytes)
        .map_err(|e| VctrlError::Other(e.to_string()))?;
    let email = String::from_utf8(email_bytes).map_err(|e| VctrlError::Other(e.to_string()))?;
    Ok(UserID { name, email })
}
