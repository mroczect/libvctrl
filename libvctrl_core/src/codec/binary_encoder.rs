use libvctrl_handler::{
    Blob, Commit, Encoder, EntryKind, MAX_MESSAGE_LENGTH, Tag, Tree, VctrlError,
};
use std::io::Write;

pub const VERSION: u8 = 3;

#[derive(Debug, Default, Clone, Copy)]
pub struct BinaryEncoder;

impl Encoder for BinaryEncoder {
    fn encode_blob<W: Write + Send>(&self, blob: &Blob, writer: &mut W) -> Result<(), VctrlError> {
        let data = blob.data();
        writer.write_all(&[VERSION]).map_err(VctrlError::from_io)?;
        writer
            .write_all(&(data.len() as u64).to_le_bytes())
            .map_err(VctrlError::from_io)?;
        writer.write_all(data).map_err(VctrlError::from_io)?;
        Ok(())
    }

    #[allow(clippy::wildcard_enum_match_arm)]
    fn encode_tree<W: Write + Send>(&self, tree: &Tree, writer: &mut W) -> Result<(), VctrlError> {
        let entries = tree.entries();
        writer.write_all(&[VERSION]).map_err(VctrlError::from_io)?;
        let entry_count = u32::try_from(entries.len())
            .map_err(|e| VctrlError::SerializationError(format!("too many entries: {e}")))?;
        writer
            .write_all(&entry_count.to_le_bytes())
            .map_err(VctrlError::from_io)?;

        for entry in entries {
            let name = entry.name();
            let name_len = u8::try_from(name.len())
                .map_err(|e| VctrlError::SerializationError(format!("name too long: {e}")))?;
            writer.write_all(&[name_len]).map_err(VctrlError::from_io)?;
            writer
                .write_all(name.as_bytes())
                .map_err(VctrlError::from_io)?;

            let kind_byte = match entry.kind() {
                EntryKind::Blob => 0,
                EntryKind::Executable => 1,
                EntryKind::Symlink => 2,
                EntryKind::Tree => 3,
                EntryKind::Submodule => 4,
                _ => return Err(VctrlError::SerializationError("unknown entry kind".into())),
            };
            writer
                .write_all(&[kind_byte])
                .map_err(VctrlError::from_io)?;
            writer
                .write_all(entry.hash().as_bytes())
                .map_err(VctrlError::from_io)?;
        }
        Ok(())
    }

    fn encode_commit<W: Write + Send>(
        &self,
        commit: &Commit,
        writer: &mut W,
    ) -> Result<(), VctrlError> {
        writer.write_all(&[VERSION]).map_err(VctrlError::from_io)?;
        writer
            .write_all(commit.tree().as_bytes())
            .map_err(VctrlError::from_io)?;

        let parents = commit.parents();
        let parent_count = u16::try_from(parents.len())
            .map_err(|e| VctrlError::SerializationError(format!("too many parents: {e}")))?;
        writer
            .write_all(&parent_count.to_le_bytes())
            .map_err(VctrlError::from_io)?;

        for parent in parents {
            writer
                .write_all(parent.as_bytes())
                .map_err(VctrlError::from_io)?;
        }

        let author_name = commit.author().name();
        writer
            .write_all(&[u8::try_from(author_name.len()).map_err(|e| {
                VctrlError::SerializationError(format!("author name too long: {e}"))
            })?])
            .map_err(VctrlError::from_io)?;
        writer
            .write_all(author_name.as_bytes())
            .map_err(VctrlError::from_io)?;

        let author_email = commit.author().email();
        writer
            .write_all(&[u8::try_from(author_email.len()).map_err(|e| {
                VctrlError::SerializationError(format!("author email too long: {e}"))
            })?])
            .map_err(VctrlError::from_io)?;
        writer
            .write_all(author_email.as_bytes())
            .map_err(VctrlError::from_io)?;

        let committer_name = commit.committer().name();
        writer
            .write_all(&[u8::try_from(committer_name.len()).map_err(|e| {
                VctrlError::SerializationError(format!("committer name too long: {e}"))
            })?])
            .map_err(VctrlError::from_io)?;
        writer
            .write_all(committer_name.as_bytes())
            .map_err(VctrlError::from_io)?;

        let committer_email = commit.committer().email();
        writer
            .write_all(&[u8::try_from(committer_email.len()).map_err(|e| {
                VctrlError::SerializationError(format!("committer email too long: {e}"))
            })?])
            .map_err(VctrlError::from_io)?;
        writer
            .write_all(committer_email.as_bytes())
            .map_err(VctrlError::from_io)?;

        let msg = commit.message();
        let msg_len = u32::try_from(msg.len())
            .map_err(|e| VctrlError::SerializationError(format!("message too long: {e}")))?;
        if msg_len as usize > usize::try_from(MAX_MESSAGE_LENGTH).unwrap_or(usize::MAX) {
            return Err(VctrlError::SerializationError(
                "commit message exceeds size limit".into(),
            ));
        }
        writer
            .write_all(&msg_len.to_le_bytes())
            .map_err(VctrlError::from_io)?;
        writer
            .write_all(msg.as_bytes())
            .map_err(VctrlError::from_io)?;

        writer
            .write_all(&commit.meta().timestamp().to_le_bytes())
            .map_err(VctrlError::from_io)?;
        writer
            .write_all(&commit.meta().timezone_offset().to_le_bytes())
            .map_err(VctrlError::from_io)?;

        match commit.meta().encoding() {
            Some(enc) => {
                let len = u8::try_from(enc.len()).map_err(|e| {
                    VctrlError::SerializationError(format!("encoding too long: {e}"))
                })?;
                writer.write_all(&[len]).map_err(VctrlError::from_io)?;
                writer
                    .write_all(enc.as_bytes())
                    .map_err(VctrlError::from_io)?;
            }
            None => writer.write_all(&[0_u8]).map_err(VctrlError::from_io)?,
        }
        Ok(())
    }

    fn encode_tag<W: Write + Send>(&self, tag: &Tag, writer: &mut W) -> Result<(), VctrlError> {
        writer.write_all(&[VERSION]).map_err(VctrlError::from_io)?;

        let name = tag.name();
        let name_len = u8::try_from(name.len())
            .map_err(|e| VctrlError::SerializationError(format!("tag name too long: {e}")))?;
        writer.write_all(&[name_len]).map_err(VctrlError::from_io)?;
        writer
            .write_all(name.as_bytes())
            .map_err(VctrlError::from_io)?;

        writer
            .write_all(tag.target().as_bytes())
            .map_err(VctrlError::from_io)?;

        match tag.tagger() {
            Some(tagger) => {
                writer.write_all(&[1_u8]).map_err(VctrlError::from_io)?;

                let tagger_name = tagger.name();
                writer
                    .write_all(&[u8::try_from(tagger_name.len()).map_err(|e| {
                        VctrlError::SerializationError(format!("tagger name too long: {e}"))
                    })?])
                    .map_err(VctrlError::from_io)?;
                writer
                    .write_all(tagger_name.as_bytes())
                    .map_err(VctrlError::from_io)?;

                let tagger_email = tagger.email();
                writer
                    .write_all(&[u8::try_from(tagger_email.len()).map_err(|e| {
                        VctrlError::SerializationError(format!("tagger email too long: {e}"))
                    })?])
                    .map_err(VctrlError::from_io)?;
                writer
                    .write_all(tagger_email.as_bytes())
                    .map_err(VctrlError::from_io)?;
            }
            None => writer.write_all(&[0_u8]).map_err(VctrlError::from_io)?,
        }

        let msg = tag.message();
        let msg_len = u32::try_from(msg.len())
            .map_err(|e| VctrlError::SerializationError(format!("message too long: {e}")))?;
        if msg_len as usize > usize::try_from(MAX_MESSAGE_LENGTH).unwrap_or(usize::MAX) {
            return Err(VctrlError::SerializationError(
                "tag message exceeds size limit".into(),
            ));
        }
        writer
            .write_all(&msg_len.to_le_bytes())
            .map_err(VctrlError::from_io)?;
        writer
            .write_all(msg.as_bytes())
            .map_err(VctrlError::from_io)?;

        writer
            .write_all(&tag.meta().timestamp().to_le_bytes())
            .map_err(VctrlError::from_io)?;
        writer
            .write_all(&tag.meta().timezone_offset().to_le_bytes())
            .map_err(VctrlError::from_io)?;

        match tag.meta().encoding() {
            Some(enc) => {
                let len = u8::try_from(enc.len()).map_err(|e| {
                    VctrlError::SerializationError(format!("encoding too long: {e}"))
                })?;
                writer.write_all(&[len]).map_err(VctrlError::from_io)?;
                writer
                    .write_all(enc.as_bytes())
                    .map_err(VctrlError::from_io)?;
            }
            None => writer.write_all(&[0_u8]).map_err(VctrlError::from_io)?,
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::BinaryDecoder;
    use libvctrl_handler::{CommitMeta, Decoder, Hash, TreeEntry, UserID};
    use std::io::Cursor;

    fn hash_byte(byte: u8) -> Result<Hash, VctrlError> {
        Hash::from_bytes(&[byte; 64])
    }

    fn user(name: &str, email: &str) -> Result<UserID, VctrlError> {
        UserID::new(name.to_string(), email.to_string())
    }

    #[test]
    fn encode_blob_exact_bytes() -> Result<(), VctrlError> {
        let blob = Blob::new(vec![1_u8, 2, 3])?;
        let mut buf = Vec::new();
        BinaryEncoder.encode_blob(&blob, &mut buf)?;
        assert_eq!(buf, vec![3, 3, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3]);
        Ok(())
    }

    #[test]
    fn encode_tree_exact_prefix() -> Result<(), VctrlError> {
        let hash = hash_byte(0x22)?;
        let entry = TreeEntry::new("a".to_string(), EntryKind::Blob, hash)?;
        let tree = Tree::new(vec![entry])?;

        let mut buf = Vec::new();
        BinaryEncoder.encode_tree(&tree, &mut buf)?;

        assert_eq!(buf.first(), Some(&3_u8));
        assert_eq!(buf.get(1..5), Some(&1_u32.to_le_bytes()[..]));
        assert_eq!(buf.get(5), Some(&1_u8));
        assert_eq!(buf.get(6), Some(&b'a'));
        assert_eq!(buf.get(7), Some(&0_u8));
        assert_eq!(buf.len(), 5 + 1 + 1 + 1 + 64);
        Ok(())
    }

    #[test]
    fn encode_commit_roundtrip_with_decoder() -> Result<(), VctrlError> {
        let tree = hash_byte(0x11)?;
        let parent = hash_byte(0x12)?;
        let author = user("Alice", "alice@example.com")?;
        let committer = user("Bob", "bob@example.com")?;
        let message = "commit message".to_string();
        let meta = CommitMeta::new(123, 0, None)?;

        let commit =
            Commit::with_meta(tree, vec![parent], author, committer, message.clone(), meta)?;

        let mut buf = Vec::new();
        BinaryEncoder.encode_commit(&commit, &mut buf)?;
        let decoded = BinaryDecoder.decode_commit(Cursor::new(buf))?;

        assert_eq!(decoded.tree(), &tree);
        assert_eq!(decoded.parents(), &[parent]);
        assert_eq!(decoded.author().name(), "Alice");
        assert_eq!(decoded.committer().email(), "bob@example.com");
        assert_eq!(decoded.message(), message);
        assert_eq!(decoded.meta().timestamp(), 123);
        assert_eq!(decoded.meta().timezone_offset(), 0);
        assert!(decoded.meta().encoding().is_none());
        Ok(())
    }

    #[test]
    fn encode_tag_roundtrip_with_decoder() -> Result<(), VctrlError> {
        let target = hash_byte(0x33)?;
        let tagger = user("Tagger", "tagger@example.com")?;
        let message = "v1.0".to_string();
        let meta = CommitMeta::new(456, 0, None)?;

        let tag = Tag::with_meta(
            "v1.0".to_string(),
            target,
            Some(tagger),
            message.clone(),
            meta,
        )?;

        let mut buf = Vec::new();
        BinaryEncoder.encode_tag(&tag, &mut buf)?;
        let decoded = BinaryDecoder.decode_tag(Cursor::new(buf))?;

        assert_eq!(decoded.name(), "v1.0");
        assert_eq!(decoded.target(), &target);
        assert_eq!(
            decoded
                .tagger()
                .ok_or_else(|| VctrlError::Other("expected tagger".into()))?
                .name(),
            "Tagger"
        );
        assert_eq!(decoded.message(), message);
        assert_eq!(decoded.meta().timestamp(), 456);
        assert_eq!(decoded.meta().timezone_offset(), 0);
        assert!(decoded.meta().encoding().is_none());
        Ok(())
    }
}
