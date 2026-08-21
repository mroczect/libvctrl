use alloc::str;
use alloc::sync::Arc;

use libvctrl_handler::{
    Blob, Commit, CommitMeta, Decoder, EntryKind, HASH_LENGTH, Hash, MAX_BLOB_SIZE,
    MAX_MESSAGE_LENGTH, MAX_TREE_ENTRIES, Tag, Tree, TreeEntry, UserID, VctrlError,
};

const EXPECTED_VERSION: u8 = 3;

#[derive(Debug, Copy, Clone)]
pub struct BinaryDecoder;

impl BinaryDecoder {
    fn check_version(data: &[u8]) -> Result<&[u8], VctrlError> {
        let version = data
            .first()
            .copied()
            .ok_or_else(|| VctrlError::CorruptedData("missing version byte".into()))?;
        if version != EXPECTED_VERSION {
            return Err(VctrlError::CorruptedData(format!(
                "unsupported version: {} (expected {})",
                version, EXPECTED_VERSION
            )));
        }
        data.get(1..)
            .ok_or_else(|| VctrlError::CorruptedData("missing payload after version".into()))
    }

    fn read_bounded<R: std::io::Read>(
        reader: &mut R,
        max_size: usize,
    ) -> Result<Vec<u8>, VctrlError> {
        let mut buf = Vec::new();
        let mut chunk = [0u8; 4096];
        loop {
            let n = reader
                .read(&mut chunk)
                .map_err(|e| VctrlError::IoError(Arc::new(e)))?;
            if n == 0 {
                break;
            }
            if buf.len() + n > max_size {
                return Err(VctrlError::CorruptedData(
                    "stream exceeds maximum allowed size".into(),
                ));
            }
            buf.extend_from_slice(chunk.get(..n).unwrap_or(&[]));
        }
        Ok(buf)
    }

    fn require_byte(data: &[u8], pos: usize, what: &str) -> Result<u8, VctrlError> {
        data.get(pos)
            .copied()
            .ok_or_else(|| VctrlError::CorruptedData(format!("missing {what}")))
    }

    fn require_slice<'a>(
        data: &'a [u8],
        start: usize,
        len: usize,
        what: &str,
    ) -> Result<&'a [u8], VctrlError> {
        let end = start
            .checked_add(len)
            .ok_or_else(|| VctrlError::CorruptedData(format!("invalid {what} length")))?;
        data.get(start..end)
            .ok_or_else(|| VctrlError::CorruptedData(format!("{what} truncated")))
    }
}

impl Decoder for BinaryDecoder {
    fn decode_blob<R: std::io::Read + Send>(&self, mut reader: R) -> Result<Blob, VctrlError> {
        let max_size = usize::try_from(MAX_BLOB_SIZE).unwrap_or(usize::MAX) + 16;
        let data = Self::read_bounded(&mut reader, max_size)?;

        let data = Self::check_version(&data)?;
        let len_bytes: [u8; 8] = Self::require_slice(data, 0, 8, "blob length prefix")?
            .try_into()
            .map_err(|e| VctrlError::CorruptedData(format!("invalid blob length prefix: {e}")))?;

        let data_len = usize::try_from(u64::from_le_bytes(len_bytes))
            .map_err(|e| VctrlError::CorruptedData(format!("blob length out of range: {e}")))?;

        if data_len > usize::try_from(MAX_BLOB_SIZE).unwrap_or(usize::MAX) {
            return Err(VctrlError::CorruptedData("blob exceeds size limit".into()));
        }

        let total_len = 8usize
            .checked_add(data_len)
            .ok_or_else(|| VctrlError::CorruptedData("blob length overflow".into()))?;
        if data.len() != total_len {
            return Err(VctrlError::CorruptedData("blob length mismatch".into()));
        }

        let payload = Self::require_slice(data, 8, data_len, "blob data")?;
        Blob::new(payload.to_vec())
    }

    fn decode_tree<R: std::io::Read + Send>(&self, mut reader: R) -> Result<Tree, VctrlError> {
        let max_size = usize::try_from(MAX_TREE_ENTRIES).unwrap_or(usize::MAX) * 321 + 5;
        let data = Self::read_bounded(&mut reader, max_size)?;

        let data = Self::check_version(&data)?;
        let count_bytes: [u8; 4] = Self::require_slice(data, 0, 4, "tree entry count")?
            .try_into()
            .map_err(|e| VctrlError::CorruptedData(format!("invalid tree count: {e}")))?;
        let count = u32::from_le_bytes(count_bytes) as usize;

        if count > usize::try_from(MAX_TREE_ENTRIES).unwrap_or(usize::MAX) {
            return Err(VctrlError::CorruptedData(
                "tree entry count exceeds limit".into(),
            ));
        }

        let mut pos = 4usize;
        let mut entries = Vec::with_capacity(count);

        for _ in 0..count {
            let name_len = Self::require_byte(data, pos, "tree entry name length")? as usize;
            pos += 1;

            let name_bytes = Self::require_slice(data, pos, name_len, "tree entry name")?;
            let name = str::from_utf8(name_bytes)
                .map_err(|e| VctrlError::CorruptedData(format!("invalid UTF-8 in name: {e}")))?
                .to_string();
            pos += name_len;

            let kind_byte = Self::require_byte(data, pos, "tree entry kind")?;
            pos += 1;

            let kind = match kind_byte {
                0 => EntryKind::Blob,
                1 => EntryKind::Executable,
                2 => EntryKind::Symlink,
                3 => EntryKind::Tree,
                4 => EntryKind::Submodule,
                other => {
                    return Err(VctrlError::CorruptedData(format!(
                        "unknown entry kind: {other}"
                    )));
                }
            };

            let hash_bytes = Self::require_slice(data, pos, HASH_LENGTH, "tree entry hash")?;
            let hash = Hash::from_bytes(hash_bytes)?;
            pos += HASH_LENGTH;

            entries.push(TreeEntry::new(name, kind, hash)?);
        }

        if pos != data.len() {
            return Err(VctrlError::CorruptedData("trailing bytes in tree".into()));
        }

        Tree::new(entries)
    }

    #[allow(clippy::too_many_lines)]
    fn decode_commit<R: std::io::Read + Send>(&self, mut reader: R) -> Result<Commit, VctrlError> {
        let max_size = usize::try_from(MAX_MESSAGE_LENGTH).unwrap_or(usize::MAX) + 1024;
        let data = Self::read_bounded(&mut reader, max_size)?;
        let data = Self::check_version(&data)?;

        let tree_hash = Self::require_slice(data, 0, HASH_LENGTH, "commit tree hash")?;
        let tree = Hash::from_bytes(tree_hash)?;

        let parent_count_bytes = Self::require_slice(data, HASH_LENGTH, 2, "commit parent count")?;
        let parent_count = u16::from_le_bytes(
            parent_count_bytes
                .try_into()
                .map_err(|e| VctrlError::CorruptedData(format!("invalid parent count: {e}")))?,
        ) as usize;

        let mut pos = HASH_LENGTH + 2;
        let mut parents = Vec::with_capacity(parent_count);
        for _ in 0..parent_count {
            let parent_bytes = Self::require_slice(data, pos, HASH_LENGTH, "parent hash")?;
            parents.push(Hash::from_bytes(parent_bytes)?);
            pos += HASH_LENGTH;
        }

        let author_name_len = Self::require_byte(data, pos, "author name length")? as usize;
        pos += 1;
        let author_name_bytes = Self::require_slice(data, pos, author_name_len, "author name")?;
        let author_name = str::from_utf8(author_name_bytes)
            .map_err(|e| VctrlError::CorruptedData(format!("invalid UTF-8 in author name: {e}")))?
            .to_string();
        pos += author_name_len;

        let author_email_len = Self::require_byte(data, pos, "author email length")? as usize;
        pos += 1;
        let author_email_bytes = Self::require_slice(data, pos, author_email_len, "author email")?;
        let author_email = str::from_utf8(author_email_bytes)
            .map_err(|e| VctrlError::CorruptedData(format!("invalid UTF-8 in author email: {e}")))?
            .to_string();
        pos += author_email_len;

        let author = UserID::new(author_name, author_email)?;

        let committer_name_len = Self::require_byte(data, pos, "committer name length")? as usize;
        pos += 1;
        let committer_name_bytes =
            Self::require_slice(data, pos, committer_name_len, "committer name")?;
        let committer_name = str::from_utf8(committer_name_bytes)
            .map_err(|e| {
                VctrlError::CorruptedData(format!("invalid UTF-8 in committer name: {e}"))
            })?
            .to_string();
        pos += committer_name_len;

        let committer_email_len = Self::require_byte(data, pos, "committer email length")? as usize;
        pos += 1;
        let committer_email_bytes =
            Self::require_slice(data, pos, committer_email_len, "committer email")?;
        let committer_email = str::from_utf8(committer_email_bytes)
            .map_err(|e| {
                VctrlError::CorruptedData(format!("invalid UTF-8 in committer email: {e}"))
            })?
            .to_string();
        pos += committer_email_len;

        let committer = UserID::new(committer_name, committer_email)?;

        let msg_len_bytes = Self::require_slice(data, pos, 4, "commit message length")?;
        let msg_len = u32::from_le_bytes(
            msg_len_bytes
                .try_into()
                .map_err(|e| VctrlError::CorruptedData(format!("invalid message length: {e}")))?,
        ) as usize;
        pos += 4;

        if msg_len > usize::try_from(MAX_MESSAGE_LENGTH).unwrap_or(usize::MAX) {
            return Err(VctrlError::CorruptedData(
                "commit message exceeds size limit".into(),
            ));
        }

        let msg_bytes = Self::require_slice(data, pos, msg_len, "commit message")?;
        let message = str::from_utf8(msg_bytes)
            .map_err(|e| VctrlError::CorruptedData(format!("invalid UTF-8 in message: {e}")))?
            .to_string();
        pos += msg_len;

        let timestamp_bytes = Self::require_slice(data, pos, 8, "commit timestamp")?;
        let timestamp = i64::from_le_bytes(
            timestamp_bytes
                .try_into()
                .map_err(|e| VctrlError::CorruptedData(format!("invalid timestamp: {e}")))?,
        );
        pos += 8;

        let tz_bytes = Self::require_slice(data, pos, 2, "commit timezone offset")?;
        let timezone_offset = i16::from_le_bytes(
            tz_bytes
                .try_into()
                .map_err(|e| VctrlError::CorruptedData(format!("invalid timezone offset: {e}")))?,
        );
        pos += 2;

        let encoding_len = Self::require_byte(data, pos, "commit encoding length")? as usize;
        pos += 1;
        let encoding = if encoding_len > 0 {
            let enc_bytes = Self::require_slice(data, pos, encoding_len, "commit encoding")?;
            let enc = str::from_utf8(enc_bytes)
                .map_err(|e| VctrlError::CorruptedData(format!("invalid UTF-8 in encoding: {e}")))?
                .to_string();
            pos += encoding_len;
            Some(enc)
        } else {
            None
        };

        if pos != data.len() {
            return Err(VctrlError::CorruptedData("trailing bytes in commit".into()));
        }

        let meta = CommitMeta::new(timestamp, timezone_offset, encoding)?;
        Commit::with_meta(tree, parents, author, committer, message, meta)
    }

    #[allow(clippy::too_many_lines)]
    fn decode_tag<R: std::io::Read + Send>(&self, mut reader: R) -> Result<Tag, VctrlError> {
        let max_size = usize::try_from(MAX_MESSAGE_LENGTH).unwrap_or(usize::MAX) + 1024;
        let data = Self::read_bounded(&mut reader, max_size)?;
        let data = Self::check_version(&data)?;

        let name_len = Self::require_byte(data, 0, "tag name length")? as usize;
        let name_bytes = Self::require_slice(data, 1, name_len, "tag name")?;
        let name = str::from_utf8(name_bytes)
            .map_err(|e| VctrlError::CorruptedData(format!("invalid UTF-8 in tag name: {e}")))?
            .to_string();
        let mut pos = 1 + name_len;

        let target_bytes = Self::require_slice(data, pos, HASH_LENGTH, "tag target hash")?;
        let target = Hash::from_bytes(target_bytes)?;
        pos += HASH_LENGTH;

        let has_tagger = match Self::require_byte(data, pos, "tagger presence byte")? {
            0 => false,
            1 => true,
            other => {
                return Err(VctrlError::CorruptedData(format!(
                    "invalid tagger presence byte: {other}"
                )));
            }
        };
        pos += 1;

        let tagger = if has_tagger {
            let tagger_name_len = Self::require_byte(data, pos, "tagger name length")? as usize;
            pos += 1;
            let tagger_name_bytes = Self::require_slice(data, pos, tagger_name_len, "tagger name")?;
            let tagger_name = str::from_utf8(tagger_name_bytes)
                .map_err(|e| {
                    VctrlError::CorruptedData(format!("invalid UTF-8 in tagger name: {e}"))
                })?
                .to_string();
            pos += tagger_name_len;

            let tagger_email_len = Self::require_byte(data, pos, "tagger email length")? as usize;
            pos += 1;
            let tagger_email_bytes =
                Self::require_slice(data, pos, tagger_email_len, "tagger email")?;
            let tagger_email = str::from_utf8(tagger_email_bytes)
                .map_err(|e| {
                    VctrlError::CorruptedData(format!("invalid UTF-8 in tagger email: {e}"))
                })?
                .to_string();
            pos += tagger_email_len;

            Some(UserID::new(tagger_name, tagger_email)?)
        } else {
            None
        };

        let msg_len_bytes = Self::require_slice(data, pos, 4, "tag message length")?;
        let msg_len = u32::from_le_bytes(
            msg_len_bytes
                .try_into()
                .map_err(|e| VctrlError::CorruptedData(format!("invalid message length: {e}")))?,
        ) as usize;
        pos += 4;

        if msg_len > usize::try_from(MAX_MESSAGE_LENGTH).unwrap_or(usize::MAX) {
            return Err(VctrlError::SerializationError(
                "tag message exceeds size limit".into(),
            ));
        }

        let msg_bytes = Self::require_slice(data, pos, msg_len, "tag message")?;
        let message = str::from_utf8(msg_bytes)
            .map_err(|e| VctrlError::CorruptedData(format!("invalid UTF-8 in message: {e}")))?
            .to_string();
        pos += msg_len;

        let timestamp_bytes = Self::require_slice(data, pos, 8, "tag timestamp")?;
        let timestamp = i64::from_le_bytes(
            timestamp_bytes
                .try_into()
                .map_err(|e| VctrlError::CorruptedData(format!("invalid timestamp: {e}")))?,
        );
        pos += 8;

        let tz_bytes = Self::require_slice(data, pos, 2, "tag timezone offset")?;
        let timezone_offset = i16::from_le_bytes(
            tz_bytes
                .try_into()
                .map_err(|e| VctrlError::CorruptedData(format!("invalid timezone offset: {e}")))?,
        );
        pos += 2;

        let encoding_len = Self::require_byte(data, pos, "tag encoding length")? as usize;
        pos += 1;
        let encoding = if encoding_len > 0 {
            let enc_bytes = Self::require_slice(data, pos, encoding_len, "tag encoding")?;
            let enc = str::from_utf8(enc_bytes)
                .map_err(|e| VctrlError::CorruptedData(format!("invalid UTF-8 in encoding: {e}")))?
                .to_string();
            pos += encoding_len;
            Some(enc)
        } else {
            None
        };

        if pos != data.len() {
            return Err(VctrlError::CorruptedData("trailing bytes in tag".into()));
        }

        let meta = CommitMeta::new(timestamp, timezone_offset, encoding)?;
        Tag::with_meta(name, target, tagger, message, meta)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::BinaryEncoder;
    use libvctrl_handler::{Encoder, TreeEntry};
    use std::io::Cursor;

    fn hash_byte(byte: u8) -> Result<Hash, VctrlError> {
        Hash::from_bytes(&[byte; 64])
    }

    fn user(name: &str, email: &str) -> Result<UserID, VctrlError> {
        UserID::new(name.to_string(), email.to_string())
    }

    fn meta(ts: i64, tz: i16) -> Result<CommitMeta, VctrlError> {
        CommitMeta::new(ts, tz, None)
    }

    #[test]
    fn check_version_valid() -> Result<(), VctrlError> {
        let data = [3_u8, 42];
        let rest = BinaryDecoder::check_version(&data)?;
        assert_eq!(rest, &[42]);
        Ok(())
    }

    #[test]
    fn check_version_missing_byte() {
        assert!(BinaryDecoder::check_version(&[]).is_err());
    }

    #[test]
    fn check_version_unsupported() -> Result<(), VctrlError> {
        let result = BinaryDecoder::check_version(&[4_u8]);
        assert!(result.is_err());
        match result {
            Err(VctrlError::CorruptedData(msg)) => {
                assert!(msg.contains("unsupported version"));
            }
            _ => return Err(VctrlError::Other("expected CorruptedData".into())),
        }
        Ok(())
    }

    #[test]
    fn read_bounded_within_limit() -> Result<(), VctrlError> {
        let mut reader = Cursor::new(vec![1_u8, 2, 3]);
        let data = BinaryDecoder::read_bounded(&mut reader, 10)?;
        assert_eq!(data, vec![1, 2, 3]);
        Ok(())
    }

    #[test]
    fn read_bounded_exceeds_limit() {
        let mut reader = Cursor::new(vec![1_u8, 2, 3, 4]);
        assert!(BinaryDecoder::read_bounded(&mut reader, 2).is_err());
    }

    #[test]
    fn require_byte_valid() -> Result<(), VctrlError> {
        let value = BinaryDecoder::require_byte(&[10, 20], 1, "second byte")?;
        assert_eq!(value, 20);
        Ok(())
    }

    #[test]
    fn require_byte_missing() {
        assert!(BinaryDecoder::require_byte(&[10], 1, "second byte").is_err());
    }

    #[test]
    fn require_slice_valid() -> Result<(), VctrlError> {
        let data = [1, 2, 3, 4];
        let slice = BinaryDecoder::require_slice(&data, 1, 2, "middle")?;
        assert_eq!(slice, &[2, 3]);
        Ok(())
    }

    #[test]
    fn require_slice_overflow() {
        let data = [1, 2, 3];
        assert!(BinaryDecoder::require_slice(&data, usize::MAX, 2, "overflow").is_err());
    }

    #[test]
    fn decode_blob_valid_roundtrip() -> Result<(), VctrlError> {
        let encoder = BinaryEncoder;
        let codec = BinaryDecoder;
        let payload = vec![1_u8, 2, 3, 4];

        let blob = Blob::new(payload.clone())?;
        let mut buf = Vec::new();
        encoder.encode_blob(&blob, &mut buf)?;
        let decoded = codec.decode_blob(Cursor::new(buf))?;
        assert_eq!(decoded.data(), payload.as_slice());
        Ok(())
    }

    #[test]
    fn decode_blob_invalid_version() {
        let codec = BinaryDecoder;
        let data = [4_u8, 0, 0, 0, 0, 0, 0, 0, 0];
        assert!(codec.decode_blob(Cursor::new(data)).is_err());
    }

    #[test]
    fn decode_blob_length_mismatch() {
        let codec = BinaryDecoder;
        let mut data = Vec::new();
        data.push(3_u8);
        data.extend_from_slice(&5_u64.to_le_bytes());
        data.push(1_u8);
        assert!(codec.decode_blob(Cursor::new(data)).is_err());
    }

    #[test]
    fn decode_tree_valid_roundtrip() -> Result<(), VctrlError> {
        let encoder = BinaryEncoder;
        let codec = BinaryDecoder;

        let hash = hash_byte(0x22)?;
        let entry = TreeEntry::new("a.txt".to_string(), EntryKind::Blob, hash)?;
        let tree = Tree::new(vec![entry])?;

        let mut buf = Vec::new();
        encoder.encode_tree(&tree, &mut buf)?;
        let decoded = codec.decode_tree(Cursor::new(buf))?;

        let entries = decoded.entries();
        assert_eq!(entries.len(), 1);
        let first = entries
            .first()
            .ok_or_else(|| VctrlError::Other("expected one entry".into()))?;
        assert_eq!(first.name(), "a.txt");
        assert_eq!(first.kind(), EntryKind::Blob);
        assert_eq!(*first.hash(), hash);
        Ok(())
    }

    #[test]
    fn decode_tree_unknown_kind() -> Result<(), VctrlError> {
        let codec = BinaryDecoder;
        let mut data = Vec::new();
        data.push(3_u8);
        data.extend_from_slice(&1_u32.to_le_bytes());
        data.push(1_u8);
        data.push(b'a');
        data.push(9_u8);
        data.extend_from_slice(hash_byte(0x33)?.as_bytes());

        let result = codec.decode_tree(Cursor::new(data));
        assert!(result.is_err());
        match result {
            Err(VctrlError::CorruptedData(msg)) => {
                assert!(msg.contains("unknown entry kind"));
            }
            _ => return Err(VctrlError::Other("expected CorruptedData".into())),
        }
        Ok(())
    }

    #[test]
    fn decode_commit_valid_roundtrip() -> Result<(), VctrlError> {
        let encoder = BinaryEncoder;
        let codec = BinaryDecoder;

        let tree = hash_byte(0x01)?;
        let parent = hash_byte(0x02)?;
        let author = user("Alice", "alice@example.com")?;
        let committer = user("Bob", "bob@example.com")?;
        let message = "initial commit".to_string();
        let meta = meta(1_600_000_000, 0)?;

        let commit =
            Commit::with_meta(tree, vec![parent], author, committer, message.clone(), meta)?;

        let mut buf = Vec::new();
        encoder.encode_commit(&commit, &mut buf)?;
        let decoded = codec.decode_commit(Cursor::new(buf))?;

        assert_eq!(decoded.tree(), &tree);
        let parents = decoded.parents();
        assert_eq!(parents.len(), 1);
        assert_eq!(parents.first(), Some(&parent));
        assert_eq!(decoded.author().name(), "Alice");
        assert_eq!(decoded.committer().email(), "bob@example.com");
        assert_eq!(decoded.message(), message);
        assert_eq!(decoded.meta().timestamp(), 1_600_000_000);
        assert_eq!(decoded.meta().timezone_offset(), 0);
        assert!(decoded.meta().encoding().is_none());
        Ok(())
    }

    #[test]
    fn decode_commit_trailing_bytes() -> Result<(), VctrlError> {
        let encoder = BinaryEncoder;
        let codec = BinaryDecoder;

        let tree = hash_byte(0x01)?;
        let author = user("Alice", "alice@example.com")?;
        let committer = user("Bob", "bob@example.com")?;
        let message = "initial commit".to_string();
        let meta = meta(1_600_000_000, 0)?;

        let commit = Commit::with_meta(tree, vec![], author, committer, message, meta)?;
        let mut buf = Vec::new();
        encoder.encode_commit(&commit, &mut buf)?;
        buf.push(0_u8);

        assert!(codec.decode_commit(Cursor::new(buf)).is_err());
        Ok(())
    }

    #[test]
    fn decode_tag_valid_roundtrip() -> Result<(), VctrlError> {
        let encoder = BinaryEncoder;
        let codec = BinaryDecoder;

        let target = hash_byte(0x33)?;
        let tagger = user("Tagger", "tagger@example.com")?;
        let message = "v1.0".to_string();
        let meta = meta(1_600_000_000, 0)?;

        let tag = Tag::with_meta(
            "v1.0".to_string(),
            target,
            Some(tagger),
            message.clone(),
            meta,
        )?;

        let mut buf = Vec::new();
        encoder.encode_tag(&tag, &mut buf)?;
        let decoded = codec.decode_tag(Cursor::new(buf))?;

        assert_eq!(decoded.name(), "v1.0");
        assert_eq!(decoded.target(), &target);
        let decoded_tagger = decoded
            .tagger()
            .ok_or_else(|| VctrlError::Other("expected tagger".into()))?;
        assert_eq!(decoded_tagger.name(), "Tagger");
        assert_eq!(decoded.message(), message);
        assert_eq!(decoded.meta().timestamp(), 1_600_000_000);
        assert_eq!(decoded.meta().timezone_offset(), 0);
        assert!(decoded.meta().encoding().is_none());
        Ok(())
    }

    #[test]
    fn decode_tag_invalid_tagger_presence() -> Result<(), VctrlError> {
        let codec = BinaryDecoder;
        let mut data = Vec::new();
        data.push(3_u8);
        data.push(1_u8);
        data.push(b'v');
        data.extend_from_slice(hash_byte(0x33)?.as_bytes());
        data.push(2_u8);

        let result = codec.decode_tag(Cursor::new(data));
        assert!(result.is_err());
        match result {
            Err(VctrlError::CorruptedData(msg)) => {
                assert!(msg.contains("invalid tagger presence"));
            }
            _ => return Err(VctrlError::Other("expected CorruptedData".into())),
        }
        Ok(())
    }
}
