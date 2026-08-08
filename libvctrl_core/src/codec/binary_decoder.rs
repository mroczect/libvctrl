//! Binary decoder – reconstructs objects from the format emitted by [`BinaryEncoder`].
//!
//! This module provides [`BinaryDecoder`], which implements the [`Decoder`] trait.
//! It carefully validates every aspect of the input to prevent panics and to
//! reject malicious or corrupted data.

use libvctrl_handler::{
    Blob, Commit, Decoder, EntryKind, Hash, MAX_BLOB_SIZE, MAX_MESSAGE_LENGTH, MAX_TREE_ENTRIES,
    Tag, Tree, TreeEntry, UserID, VctrlError,
};
use std::str;

/// Expected version byte for the current binary format.
const EXPECTED_VERSION: u8 = 1;

/// Decodes objects from the binary format produced by [`BinaryEncoder`](super::binary_encoder::BinaryEncoder).
///
/// # Safety and validation
///
/// The decoder is the **first line of defence** against corrupted or malicious
/// data. It performs strict checks on every field:
/// - The first byte must match the expected version (`0x01`).
/// - Length prefixes must match the actual data length.
/// - Names must be valid UTF‑8 and within the allowed length.
/// - Entry kinds must be known values.
/// - The number of tree entries, blob size, and message length must not exceed
///   the limits defined in `libvctrl_handler::constants`.
/// - Hashes must be exactly 64 bytes.
///
/// If any check fails, [`VctrlError::CorruptedData`] is returned immediately.
/// This ensures that invalid data is never silently accepted.
///
/// # Error handling
///
/// All methods return a `Result`. The only error variant used is
/// [`VctrlError::CorruptedData`] (and occasionally [`VctrlError::InvalidName`]
/// from constructors), because every problem at this level is a sign of
/// corrupted or malicious input.
///
/// # Example
///
/// ```rust
/// use libvctrl_core::codec::BinaryDecoder;
/// use libvctrl_handler::{Blob, Decoder};
///
/// let decoder = BinaryDecoder;
/// // A valid blob encoding: version byte 1, length 5, data "hello"
/// let valid_input = [1, 5, 0, 0, 0, 0, 0, 0, 0, b'h', b'e', b'l', b'l', b'o'];
/// let blob = decoder.decode_blob(&valid_input).expect("valid blob");
/// assert_eq!(blob.data(), b"hello");
///
/// // Truncated input (missing version byte)
/// let no_version = [5, 0, 0, 0, 0, 0, 0, 0, b'h'];
/// assert!(decoder.decode_blob(&no_version).is_err());
/// ```
pub struct BinaryDecoder;

impl BinaryDecoder {
    /// Check that the input starts with the expected version byte,
    /// returning the slice without that byte.
    fn check_version(data: &[u8]) -> Result<&[u8], VctrlError> {
        if data.is_empty() {
            return Err(VctrlError::CorruptedData("missing version byte".into()));
        }
        if data[0] != EXPECTED_VERSION {
            return Err(VctrlError::CorruptedData(format!(
                "unsupported version: {} (expected {})",
                data[0], EXPECTED_VERSION
            )));
        }
        Ok(&data[1..])
    }
}

impl Decoder for BinaryDecoder {
    fn decode_blob(&self, data: &[u8]) -> Result<Blob, VctrlError> {
        let data = Self::check_version(data)?;
        if data.len() < 8 {
            return Err(VctrlError::CorruptedData(
                "blob too short for length prefix".into(),
            ));
        }
        let len_bytes: [u8; 8] = data[..8].try_into().unwrap();
        let data_len = usize::try_from(u64::from_le_bytes(len_bytes))
            .map_err(|_| VctrlError::CorruptedData("blob length out of range".into()))?;
        if data_len > MAX_BLOB_SIZE {
            return Err(VctrlError::CorruptedData("blob exceeds size limit".into()));
        }
        if data.len() != 8 + data_len {
            return Err(VctrlError::CorruptedData("blob length mismatch".into()));
        }
        Ok(Blob::new(data[8..].to_vec()))
    }

    fn decode_tree(&self, data: &[u8]) -> Result<Tree, VctrlError> {
        let data = Self::check_version(data)?;
        if data.len() < 4 {
            return Err(VctrlError::CorruptedData("tree too short".into()));
        }
        let count_bytes: [u8; 4] = data[..4].try_into().unwrap();
        let count = u32::from_le_bytes(count_bytes) as usize;
        if count > MAX_TREE_ENTRIES {
            return Err(VctrlError::CorruptedData(
                "tree entry count exceeds limit".into(),
            ));
        }
        let mut pos = 4;
        let mut entries = Vec::with_capacity(count);
        for _ in 0..count {
            if pos >= data.len() {
                return Err(VctrlError::CorruptedData("unexpected end of tree".into()));
            }
            let name_len = data[pos] as usize;
            pos += 1;
            if pos + name_len > data.len() {
                return Err(VctrlError::CorruptedData("name exceeds data".into()));
            }
            let name = str::from_utf8(&data[pos..pos + name_len])
                .map_err(|_| VctrlError::CorruptedData("invalid UTF-8 in name".into()))?
                .to_string();
            pos += name_len;
            if pos >= data.len() {
                return Err(VctrlError::CorruptedData("missing kind".into()));
            }
            let kind = match data[pos] {
                0 => EntryKind::Blob,
                1 => EntryKind::Tree,
                _ => return Err(VctrlError::CorruptedData("unknown entry kind".into())),
            };
            pos += 1;
            if pos + 64 > data.len() {
                return Err(VctrlError::CorruptedData("hash truncated".into()));
            }
            let hash = Hash::from_bytes(&data[pos..pos + 64])?;
            pos += 64;
            entries.push(TreeEntry::new(name, kind, hash)?);
        }
        Tree::new(entries)
    }

    fn decode_commit(&self, data: &[u8]) -> Result<Commit, VctrlError> {
        let data = Self::check_version(data)?;
        if data.len() < 64 + 1 {
            return Err(VctrlError::CorruptedData("commit too short".into()));
        }
        let tree = Hash::from_bytes(&data[..64])?;
        let parent_count = data[64] as usize;
        let mut pos = 65;
        let mut parents = Vec::with_capacity(parent_count);
        for _ in 0..parent_count {
            if pos + 64 > data.len() {
                return Err(VctrlError::CorruptedData("parent hash truncated".into()));
            }
            parents.push(Hash::from_bytes(&data[pos..pos + 64])?);
            pos += 64;
        }
        // Author name
        if pos >= data.len() {
            return Err(VctrlError::CorruptedData("missing author name".into()));
        }
        let author_name_len = data[pos] as usize;
        pos += 1;
        if pos + author_name_len > data.len() {
            return Err(VctrlError::CorruptedData("author name truncated".into()));
        }
        let author_name = str::from_utf8(&data[pos..pos + author_name_len])
            .map_err(|_| VctrlError::CorruptedData("invalid UTF-8 in author name".into()))?
            .to_string();
        pos += author_name_len;
        // Author email
        if pos >= data.len() {
            return Err(VctrlError::CorruptedData("missing author email".into()));
        }
        let author_email_len = data[pos] as usize;
        pos += 1;
        if pos + author_email_len > data.len() {
            return Err(VctrlError::CorruptedData("author email truncated".into()));
        }
        let author_email = str::from_utf8(&data[pos..pos + author_email_len])
            .map_err(|_| VctrlError::CorruptedData("invalid UTF-8 in author email".into()))?
            .to_string();
        pos += author_email_len;
        let author = UserID::new(author_name, author_email)?;
        // Committer name
        if pos >= data.len() {
            return Err(VctrlError::CorruptedData("missing committer name".into()));
        }
        let committer_name_len = data[pos] as usize;
        pos += 1;
        if pos + committer_name_len > data.len() {
            return Err(VctrlError::CorruptedData("committer name truncated".into()));
        }
        let committer_name = str::from_utf8(&data[pos..pos + committer_name_len])
            .map_err(|_| VctrlError::CorruptedData("invalid UTF-8 in committer name".into()))?
            .to_string();
        pos += committer_name_len;
        // Committer email
        if pos >= data.len() {
            return Err(VctrlError::CorruptedData("missing committer email".into()));
        }
        let committer_email_len = data[pos] as usize;
        pos += 1;
        if pos + committer_email_len > data.len() {
            return Err(VctrlError::CorruptedData(
                "committer email truncated".into(),
            ));
        }
        let committer_email = str::from_utf8(&data[pos..pos + committer_email_len])
            .map_err(|_| VctrlError::CorruptedData("invalid UTF-8 in committer email".into()))?
            .to_string();
        pos += committer_email_len;
        let committer = UserID::new(committer_name, committer_email)?;
        // Message
        if pos + 4 > data.len() {
            return Err(VctrlError::CorruptedData("missing message length".into()));
        }
        let msg_len_bytes: [u8; 4] = data[pos..pos + 4].try_into().unwrap();
        let msg_len = u32::from_le_bytes(msg_len_bytes) as usize;
        pos += 4;
        if msg_len > MAX_MESSAGE_LENGTH {
            return Err(VctrlError::CorruptedData(
                "commit message exceeds size limit".into(),
            ));
        }
        if pos + msg_len > data.len() {
            return Err(VctrlError::CorruptedData("message truncated".into()));
        }
        let message = str::from_utf8(&data[pos..pos + msg_len])
            .map_err(|_| VctrlError::CorruptedData("invalid UTF-8 in message".into()))?
            .to_string();
        Ok(Commit::new(tree, parents, author, committer, message))
    }

    fn decode_tag(&self, data: &[u8]) -> Result<Tag, VctrlError> {
        let data = Self::check_version(data)?;
        if data.is_empty() {
            return Err(VctrlError::CorruptedData("tag too short".into()));
        }
        let name_len = data[0] as usize;
        let mut pos = 1;
        if pos + name_len > data.len() {
            return Err(VctrlError::CorruptedData("tag name truncated".into()));
        }
        let name = str::from_utf8(&data[pos..pos + name_len])
            .map_err(|_| VctrlError::CorruptedData("invalid UTF-8 in tag name".into()))?
            .to_string();
        pos += name_len;
        if pos + 64 > data.len() {
            return Err(VctrlError::CorruptedData("target hash truncated".into()));
        }
        let target = Hash::from_bytes(&data[pos..pos + 64])?;
        pos += 64;
        if pos >= data.len() {
            return Err(VctrlError::CorruptedData(
                "missing tagger presence byte".into(),
            ));
        }
        let has_tagger = data[pos] == 1;
        pos += 1;
        let tagger = if has_tagger {
            if pos >= data.len() {
                return Err(VctrlError::CorruptedData("missing tagger name".into()));
            }
            let tagger_name_len = data[pos] as usize;
            pos += 1;
            if pos + tagger_name_len > data.len() {
                return Err(VctrlError::CorruptedData("tagger name truncated".into()));
            }
            let tagger_name = str::from_utf8(&data[pos..pos + tagger_name_len])
                .map_err(|_| VctrlError::CorruptedData("invalid UTF-8 in tagger name".into()))?
                .to_string();
            pos += tagger_name_len;
            if pos >= data.len() {
                return Err(VctrlError::CorruptedData("missing tagger email".into()));
            }
            let tagger_email_len = data[pos] as usize;
            pos += 1;
            if pos + tagger_email_len > data.len() {
                return Err(VctrlError::CorruptedData("tagger email truncated".into()));
            }
            let tagger_email = str::from_utf8(&data[pos..pos + tagger_email_len])
                .map_err(|_| VctrlError::CorruptedData("invalid UTF-8 in tagger email".into()))?
                .to_string();
            pos += tagger_email_len;
            Some(UserID::new(tagger_name, tagger_email)?)
        } else {
            None
        };
        if pos + 4 > data.len() {
            return Err(VctrlError::CorruptedData("missing message length".into()));
        }
        let msg_len_bytes: [u8; 4] = data[pos..pos + 4].try_into().unwrap();
        let msg_len = u32::from_le_bytes(msg_len_bytes) as usize;
        pos += 4;
        if msg_len > MAX_MESSAGE_LENGTH {
            return Err(VctrlError::CorruptedData(
                "tag message exceeds size limit".into(),
            ));
        }
        if pos + msg_len > data.len() {
            return Err(VctrlError::CorruptedData("message truncated".into()));
        }
        let message = str::from_utf8(&data[pos..pos + msg_len])
            .map_err(|_| VctrlError::CorruptedData("invalid UTF-8 in message".into()))?
            .to_string();
        Tag::new(name, target, tagger, message)
    }
}
