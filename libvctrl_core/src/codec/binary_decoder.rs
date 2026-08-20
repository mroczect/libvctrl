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
    use std::io::Cursor;

    fn hash_bytes(fill: u8) -> Vec<u8> {
        vec![fill; HASH_LENGTH]
    }

    #[test]
    fn test_check_version_missing_byte() {
        let result = BinaryDecoder::check_version(&[]);
        assert!(result.is_err(), "empty data should fail");
    }

    #[test]
    fn test_check_version_wrong_version() {
        let result = BinaryDecoder::check_version(&[0u8, 0xAA]);
        assert!(result.is_err(), "wrong version should fail");
    }

    #[test]
    fn test_check_version_no_payload() {
        let result = BinaryDecoder::check_version(&[EXPECTED_VERSION]);
        assert!(
            result.is_err(),
            "version byte only (no payload) should fail"
        );
    }

    #[test]
    fn test_check_version_valid() {
        let data = [EXPECTED_VERSION, 0xAA, 0xBB, 0xCC];
        let result = BinaryDecoder::check_version(&data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), &[0xAA, 0xBB, 0xCC]);
    }

    #[test]
    fn test_read_bounded_within_limit() {
        let data = vec![0x42u8; 50];
        let mut cursor = Cursor::new(data.as_slice());
        let result = BinaryDecoder::read_bounded(&mut cursor, 100);
        assert!(result.is_ok());
        let buf = result.unwrap();
        assert_eq!(buf.len(), 50);
        assert!(buf.iter().all(|&b| b == 0x42));
    }

    #[test]
    fn test_read_bounded_exceeds_limit() {
        let data = vec![0u8; 100];
        let mut cursor = Cursor::new(data.as_slice());
        let result = BinaryDecoder::read_bounded(&mut cursor, 50);
        assert!(result.is_err(), "should error when stream exceeds max size");
    }

    #[test]
    fn test_read_bounded_empty_stream() {
        let data: Vec<u8> = Vec::new();
        let mut cursor = Cursor::new(data.as_slice());
        let result = BinaryDecoder::read_bounded(&mut cursor, 100);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_require_byte_valid() {
        let data = [10, 20, 30];
        let result = BinaryDecoder::require_byte(&data, 1, "test byte");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 20);
    }

    #[test]
    fn test_require_byte_out_of_bounds() {
        let data = [10];
        let result = BinaryDecoder::require_byte(&data, 5, "test byte");
        assert!(result.is_err());
    }

    #[test]
    fn test_require_slice_valid() {
        let data = [1, 2, 3, 4, 5];
        let result = BinaryDecoder::require_slice(&data, 1, 3, "test slice");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), &[2, 3, 4]);
    }

    #[test]
    fn test_require_slice_zero_length() {
        let data = [1, 2, 3];
        let result = BinaryDecoder::require_slice(&data, 0, 0, "empty");
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_require_slice_truncated() {
        let data = [1, 2];
        let result = BinaryDecoder::require_slice(&data, 0, 5, "test slice");
        assert!(result.is_err());
    }

    #[test]
    fn test_require_slice_overflow() {
        let data = [1, 2];
        let result = BinaryDecoder::require_slice(&data, usize::MAX, 2, "overflow slice");
        assert!(
            result.is_err(),
            "should error on usize overflow in start+len"
        );
    }

    #[test]
    fn test_decode_blob_valid() {
        let payload = b"hello world";
        let mut data = Vec::new();
        data.push(EXPECTED_VERSION);
        data.extend_from_slice(&(payload.len() as u64).to_le_bytes());
        data.extend_from_slice(payload);

        let result = BinaryDecoder.decode_blob(Cursor::new(data));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data(), payload.as_slice());
    }

    #[test]
    fn test_decode_blob_empty_payload() {
        let mut data = Vec::new();
        data.push(EXPECTED_VERSION);
        data.extend_from_slice(&0u64.to_le_bytes());

        let result = BinaryDecoder.decode_blob(Cursor::new(data));
        assert!(result.is_ok());
        assert!(result.unwrap().data().is_empty());
    }

    #[test]
    fn test_decode_blob_empty_input() {
        let result = BinaryDecoder.decode_blob(Cursor::new(Vec::<u8>::new()));
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_blob_wrong_version() {
        let mut data = Vec::new();
        data.push(0);
        data.extend_from_slice(&5u64.to_le_bytes());
        data.extend_from_slice(b"hello");

        let result = BinaryDecoder.decode_blob(Cursor::new(data));
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_blob_length_mismatch_too_short() {
        let mut data = Vec::new();
        data.push(EXPECTED_VERSION);
        data.extend_from_slice(&100u64.to_le_bytes());
        data.extend_from_slice(b"short");

        let result = BinaryDecoder.decode_blob(Cursor::new(data));
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_blob_length_mismatch_too_long() {
        let mut data = Vec::new();
        data.push(EXPECTED_VERSION);
        data.extend_from_slice(&2u64.to_le_bytes());
        data.extend_from_slice(b"this is longer than 2");

        let result = BinaryDecoder.decode_blob(Cursor::new(data));
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_tree_valid_single_entry() {
        let hb = hash_bytes(0xAB);
        let mut data = Vec::new();
        data.push(EXPECTED_VERSION);
        data.extend_from_slice(&1u32.to_le_bytes());
        data.push(4);
        data.extend_from_slice(b"file");
        data.push(0);
        data.extend_from_slice(&hb);

        let result = BinaryDecoder.decode_tree(Cursor::new(data));
        assert!(result.is_ok());
        let tree = result.unwrap();
        assert_eq!(tree.entries().len(), 1);
        assert_eq!(tree.entries()[0].name(), "file");
        assert_eq!(tree.entries()[0].kind(), EntryKind::Blob);
    }

    #[test]
    fn test_decode_tree_empty() {
        let mut data = Vec::new();
        data.push(EXPECTED_VERSION);
        data.extend_from_slice(&0u32.to_le_bytes());

        let result = BinaryDecoder.decode_tree(Cursor::new(data));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().entries().len(), 0);
    }

    #[test]
    fn test_decode_tree_multiple_entries() {
        let hb1 = hash_bytes(0x01);
        let hb2 = hash_bytes(0x02);
        let mut data = Vec::new();
        data.push(EXPECTED_VERSION);
        data.extend_from_slice(&2u32.to_le_bytes());
        data.push(3);
        data.extend_from_slice(b"src");
        data.push(3);
        data.extend_from_slice(&hb1);
        data.push(9);
        data.extend_from_slice(b"Cargo.toml");
        data.push(0);
        data.extend_from_slice(&hb2);

        let result = BinaryDecoder.decode_tree(Cursor::new(data));
        assert!(result.is_ok());
        let tree = result.unwrap();
        assert_eq!(tree.entries().len(), 2);
        assert_eq!(tree.entries()[0].name(), "src");
        assert_eq!(tree.entries()[0].kind(), EntryKind::Tree);
        assert_eq!(tree.entries()[1].name(), "Cargo.toml");
        assert_eq!(tree.entries()[1].kind(), EntryKind::Blob);
    }

    #[test]
    fn test_decode_tree_unknown_kind() {
        let hb = hash_bytes(0x00);
        let mut data = Vec::new();
        data.push(EXPECTED_VERSION);
        data.extend_from_slice(&1u32.to_le_bytes());
        data.push(1);
        data.push(b'x');
        data.push(99);
        data.extend_from_slice(&hb);

        let result = BinaryDecoder.decode_tree(Cursor::new(data));
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_tree_trailing_bytes() {
        let hb = hash_bytes(0x00);
        let mut data = Vec::new();
        data.push(EXPECTED_VERSION);
        data.extend_from_slice(&1u32.to_le_bytes());
        data.push(1);
        data.push(b'x');
        data.push(0);
        data.extend_from_slice(&hb);
        data.push(0xFF);

        let result = BinaryDecoder.decode_tree(Cursor::new(data));
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_tree_all_known_kinds() {
        let kinds = [0u8, 1, 2, 3, 4];
        let mut data = Vec::new();
        data.push(EXPECTED_VERSION);
        data.extend_from_slice(&(kinds.len() as u32).to_le_bytes());
        for (i, &kind) in kinds.iter().enumerate() {
            let name = format!("entry_{i}");
            data.push(name.len() as u8);
            data.extend_from_slice(name.as_bytes());
            data.push(kind);
            data.extend_from_slice(&hash_bytes(i as u8));
        }

        let result = BinaryDecoder.decode_tree(Cursor::new(data));
        assert!(result.is_ok(), "should decode all known entry kinds");
    }

    fn build_valid_commit_bytes(
        tree_fill: u8,
        parents: &[u8],
        author_name: &str,
        author_email: &str,
        committer_name: &str,
        committer_email: &str,
        message: &str,
        timestamp: i64,
        tz: i16,
        encoding: Option<&str>,
    ) -> Vec<u8> {
        let mut data = Vec::new();
        data.push(EXPECTED_VERSION);
        data.extend_from_slice(&hash_bytes(tree_fill));
        data.extend_from_slice(&(parents.len() as u16).to_le_bytes());
        for &p in parents {
            data.extend_from_slice(&hash_bytes(p));
        }
        data.push(author_name.len() as u8);
        data.extend_from_slice(author_name.as_bytes());
        data.push(author_email.len() as u8);
        data.extend_from_slice(author_email.as_bytes());
        data.push(committer_name.len() as u8);
        data.extend_from_slice(committer_name.as_bytes());
        data.push(committer_email.len() as u8);
        data.extend_from_slice(committer_email.as_bytes());
        data.extend_from_slice(&(message.len() as u32).to_le_bytes());
        data.extend_from_slice(message.as_bytes());
        data.extend_from_slice(&timestamp.to_le_bytes());
        data.extend_from_slice(&tz.to_le_bytes());
        match encoding {
            Some(enc) => {
                data.push(enc.len() as u8);
                data.extend_from_slice(enc.as_bytes());
            }
            None => data.push(0),
        }
        data
    }

    #[test]
    fn test_decode_commit_valid_no_parents() {
        let data = build_valid_commit_bytes(
            0x01,
            &[],
            "Alice",
            "a@b.c",
            "Bob",
            "b@c.d",
            "init",
            1_700_000_000,
            0,
            None,
        );
        let result = BinaryDecoder.decode_commit(Cursor::new(data));
        assert!(result.is_ok());
        let commit = result.unwrap();
        assert_eq!(commit.parents().len(), 0);
        assert_eq!(commit.author().name(), "Alice");
        assert_eq!(commit.author().email(), "a@b.c");
        assert_eq!(commit.committer().name(), "Bob");
        assert_eq!(commit.committer().email(), "b@c.d");
        assert_eq!(commit.message(), "init");
        assert_eq!(commit.meta().timestamp(), 1_700_000_000);
        assert_eq!(commit.meta().timezone_offset(), 0);
        assert!(commit.meta().encoding().is_none());
    }

    #[test]
    fn test_decode_commit_with_parents_and_encoding() {
        let data = build_valid_commit_bytes(
            0x01,
            &[0x02, 0x03],
            "Alice",
            "alice@ex.com",
            "Bob",
            "bob@ex.com",
            "merge",
            1_700_000_000,
            3600,
            Some("UTF-8"),
        );
        let result = BinaryDecoder.decode_commit(Cursor::new(data));
        assert!(result.is_ok());
        let commit = result.unwrap();
        assert_eq!(commit.parents().len(), 2);
        assert_eq!(commit.meta().timezone_offset(), 3600);
        assert_eq!(commit.meta().encoding(), Some("UTF-8"));
        assert_eq!(commit.message(), "merge");
    }

    #[test]
    fn test_decode_commit_trailing_bytes() {
        let mut data =
            build_valid_commit_bytes(0x01, &[], "A", "a@b.c", "B", "b@c.d", "m", 0, 0, None);
        data.push(0xFF);
        let result = BinaryDecoder.decode_commit(Cursor::new(data));
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_commit_wrong_version() {
        let mut data =
            build_valid_commit_bytes(0x01, &[], "A", "a@b.c", "B", "b@c.d", "m", 0, 0, None);
        data[0] = 0;
        let result = BinaryDecoder.decode_commit(Cursor::new(data));
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_commit_empty_message() {
        let data =
            build_valid_commit_bytes(0x01, &[], "A", "a@b.c", "B", "b@c.d", "", 100, 0, None);
        let result = BinaryDecoder.decode_commit(Cursor::new(data));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().message(), "");
    }

    fn build_valid_tag_bytes(
        name: &str,
        target_fill: u8,
        tagger: Option<(&str, &str)>,
        message: &str,
        timestamp: i64,
        tz: i16,
        encoding: Option<&str>,
    ) -> Vec<u8> {
        let mut data = Vec::new();
        data.push(EXPECTED_VERSION);
        data.push(name.len() as u8);
        data.extend_from_slice(name.as_bytes());
        data.extend_from_slice(&hash_bytes(target_fill));
        match tagger {
            Some((tname, temail)) => {
                data.push(1);
                data.push(tname.len() as u8);
                data.extend_from_slice(tname.as_bytes());
                data.push(temail.len() as u8);
                data.extend_from_slice(temail.as_bytes());
            }
            None => data.push(0),
        }
        data.extend_from_slice(&(message.len() as u32).to_le_bytes());
        data.extend_from_slice(message.as_bytes());
        data.extend_from_slice(&timestamp.to_le_bytes());
        data.extend_from_slice(&tz.to_le_bytes());
        match encoding {
            Some(enc) => {
                data.push(enc.len() as u8);
                data.extend_from_slice(enc.as_bytes());
            }
            None => data.push(0),
        }
        data
    }

    #[test]
    fn test_decode_tag_valid_with_tagger() {
        let data = build_valid_tag_bytes(
            "v1.0",
            0x10,
            Some(("Alice", "alice@ex.com")),
            "release",
            1_700_000_000,
            0,
            None,
        );
        let result = BinaryDecoder.decode_tag(Cursor::new(data));
        assert!(result.is_ok());
        let tag = result.unwrap();
        assert_eq!(tag.name(), "v1.0");
        assert!(tag.tagger().is_some());
        assert_eq!(tag.tagger().unwrap().name(), "Alice");
        assert_eq!(tag.tagger().unwrap().email(), "alice@ex.com");
        assert_eq!(tag.message(), "release");
    }

    #[test]
    fn test_decode_tag_no_tagger() {
        let data = build_valid_tag_bytes("v2.0", 0x20, None, "", 1_700_000_000, 0, None);
        let result = BinaryDecoder.decode_tag(Cursor::new(data));
        assert!(result.is_ok());
        let tag = result.unwrap();
        assert_eq!(tag.name(), "v2.0");
        assert!(tag.tagger().is_none());
        assert_eq!(tag.message(), "");
    }

    #[test]
    fn test_decode_tag_with_encoding() {
        let data = build_valid_tag_bytes(
            "v3.0",
            0x30,
            Some(("Bob", "bob@ex.com")),
            "annotated",
            1_700_000_000,
            -3600,
            Some("UTF-8"),
        );
        let result = BinaryDecoder.decode_tag(Cursor::new(data));
        assert!(result.is_ok());
        let tag = result.unwrap();
        assert_eq!(tag.meta().timezone_offset(), -3600);
        assert_eq!(tag.meta().encoding(), Some("UTF-8"));
    }

    #[test]
    fn test_decode_tag_invalid_tagger_presence() {
        let data = build_valid_tag_bytes("v4.0", 0x40, None, "", 0, 0, None);
        let pos = 1 + 4 + HASH_LENGTH;
        let mut mutable_data = data;
        mutable_data[pos] = 5;
        let result = BinaryDecoder.decode_tag(Cursor::new(mutable_data));
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_tag_trailing_bytes() {
        let mut data = build_valid_tag_bytes("v5.0", 0x50, None, "", 0, 0, None);
        data.push(0xFF);
        let result = BinaryDecoder.decode_tag(Cursor::new(data));
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_tag_wrong_version() {
        let mut data = build_valid_tag_bytes("v6.0", 0x60, None, "", 0, 0, None);
        data[0] = 99;
        let result = BinaryDecoder.decode_tag(Cursor::new(data));
        assert!(result.is_err());
    }
}
