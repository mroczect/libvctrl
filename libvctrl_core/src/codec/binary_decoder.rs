


























use libvctrl_handler::{
    Blob, Commit, CommitMeta, Decoder, EntryKind, HASH_LENGTH, Hash, MAX_BLOB_SIZE,
    MAX_MESSAGE_LENGTH, MAX_TREE_ENTRIES, Tag, Tree, TreeEntry, UserID, VctrlError,
};
use std::str;


const EXPECTED_VERSION: u8 = 3;





























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
                .map_err(|e| VctrlError::IoError(std::sync::Arc::new(e)))?;
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
