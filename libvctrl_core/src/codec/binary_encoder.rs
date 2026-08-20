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
                _ => {
                    return Err(VctrlError::SerializationError("unknown entry kind".into()));
                }
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
            None => writer.write_all(&[0u8]).map_err(VctrlError::from_io)?,
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
                writer.write_all(&[1u8]).map_err(VctrlError::from_io)?;

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
            None => writer.write_all(&[0u8]).map_err(VctrlError::from_io)?,
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
            None => writer.write_all(&[0u8]).map_err(VctrlError::from_io)?,
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::BinaryDecoder;
    use libvctrl_handler::{CommitMeta, Decoder, HASH_LENGTH, Hash, TreeEntry, UserID};
    use std::io::Cursor;

    fn make_hash(fill: u8) -> Hash {
        Hash::from_bytes(&vec![fill; HASH_LENGTH]).unwrap()
    }

    fn hash_bytes(fill: u8) -> Vec<u8> {
        vec![fill; HASH_LENGTH]
    }

    fn make_user_id(name: &str, email: &str) -> UserID {
        UserID::new(name.into(), email.into()).unwrap()
    }

    fn make_meta(ts: i64, tz: i16, enc: Option<&str>) -> CommitMeta {
        CommitMeta::new(ts, tz, enc.map(|s| s.into())).unwrap()
    }

    #[test]
    fn test_encode_blob() {
        let blob = Blob::new(vec![0x01, 0x02, 0x03]).unwrap();
        let mut buf = Cursor::new(Vec::new());
        BinaryEncoder.encode_blob(&blob, &mut buf).unwrap();
        let encoded = buf.into_inner();

        let mut expected = Vec::new();
        expected.push(VERSION);
        expected.extend_from_slice(&3u64.to_le_bytes());
        expected.extend_from_slice(&[0x01, 0x02, 0x03]);
        assert_eq!(encoded, expected);
    }

    #[test]
    fn test_encode_blob_empty_data() {
        let blob = Blob::new(vec![]).unwrap();
        let mut buf = Cursor::new(Vec::new());
        BinaryEncoder.encode_blob(&blob, &mut buf).unwrap();
        let encoded = buf.into_inner();

        let mut expected = Vec::new();
        expected.push(VERSION);
        expected.extend_from_slice(&0u64.to_le_bytes());
        assert_eq!(encoded, expected);
    }

    #[test]
    fn test_encode_tree_single_entry() {
        let hash = make_hash(0xAB);
        let entry = TreeEntry::new("README".into(), EntryKind::Blob, hash).unwrap();
        let tree = Tree::new(vec![entry]).unwrap();
        let mut buf = Cursor::new(Vec::new());
        BinaryEncoder.encode_tree(&tree, &mut buf).unwrap();
        let encoded = buf.into_inner();

        let mut expected = Vec::new();
        expected.push(VERSION);
        expected.extend_from_slice(&1u32.to_le_bytes());
        expected.push(6);
        expected.extend_from_slice(b"README");
        expected.push(0);
        expected.extend_from_slice(&hash_bytes(0xAB));
        assert_eq!(encoded, expected);
    }

    #[test]
    fn test_encode_tree_empty() {
        let tree = Tree::new(vec![]).unwrap();
        let mut buf = Cursor::new(Vec::new());
        BinaryEncoder.encode_tree(&tree, &mut buf).unwrap();
        let encoded = buf.into_inner();

        let mut expected = Vec::new();
        expected.push(VERSION);
        expected.extend_from_slice(&0u32.to_le_bytes());
        assert_eq!(encoded, expected);
    }

    #[test]
    fn test_encode_tree_multiple_entries() {
        let e1 = TreeEntry::new("src".into(), EntryKind::Tree, make_hash(0x01)).unwrap();
        let e2 = TreeEntry::new("run".into(), EntryKind::Executable, make_hash(0x02)).unwrap();
        let tree = Tree::new(vec![e1, e2]).unwrap();
        let mut buf = Cursor::new(Vec::new());
        BinaryEncoder.encode_tree(&tree, &mut buf).unwrap();
        let encoded = buf.into_inner();

        let mut expected = Vec::new();
        expected.push(VERSION);
        expected.extend_from_slice(&2u32.to_le_bytes());
        expected.push(3);
        expected.extend_from_slice(b"src");
        expected.push(3);
        expected.extend_from_slice(&hash_bytes(0x01));
        expected.push(3);
        expected.extend_from_slice(b"run");
        expected.push(1);
        expected.extend_from_slice(&hash_bytes(0x02));
        assert_eq!(encoded, expected);
    }

    #[test]
    fn test_encode_commit() {
        let commit = Commit::with_meta(
            make_hash(0x01),
            vec![],
            make_user_id("Alice", "a@b.c"),
            make_user_id("Bob", "b@c.d"),
            "init".into(),
            make_meta(1_700_000_000, 0, None),
        )
        .unwrap();
        let mut buf = Cursor::new(Vec::new());
        BinaryEncoder.encode_commit(&commit, &mut buf).unwrap();
        let encoded = buf.into_inner();

        let mut expected = Vec::new();
        expected.push(VERSION);
        expected.extend_from_slice(&hash_bytes(0x01));
        expected.extend_from_slice(&0u16.to_le_bytes());
        expected.push(5);
        expected.extend_from_slice(b"Alice");
        expected.push(5);
        expected.extend_from_slice(b"a@b.c");
        expected.push(3);
        expected.extend_from_slice(b"Bob");
        expected.push(5);
        expected.extend_from_slice(b"b@c.d");
        expected.extend_from_slice(&4u32.to_le_bytes());
        expected.extend_from_slice(b"init");
        expected.extend_from_slice(&1_700_000_000_i64.to_le_bytes());
        expected.extend_from_slice(&0i16.to_le_bytes());
        expected.push(0);
        assert_eq!(encoded, expected);
    }

    #[test]
    fn test_encode_commit_with_encoding() {
        let commit = Commit::with_meta(
            make_hash(0x01),
            vec![],
            make_user_id("A", "a@b.c"),
            make_user_id("B", "b@c.d"),
            "msg".into(),
            make_meta(1_700_000_000, 3600, Some("UTF-8")),
        )
        .unwrap();
        let mut buf = Cursor::new(Vec::new());
        BinaryEncoder.encode_commit(&commit, &mut buf).unwrap();
        let encoded = buf.into_inner();

        let mut expected = Vec::new();
        expected.push(VERSION);
        expected.extend_from_slice(&hash_bytes(0x01));
        expected.extend_from_slice(&0u16.to_le_bytes());
        expected.push(1);
        expected.extend_from_slice(b"A");
        expected.push(5);
        expected.extend_from_slice(b"a@b.c");
        expected.push(1);
        expected.extend_from_slice(b"B");
        expected.push(5);
        expected.extend_from_slice(b"b@c.d");
        expected.extend_from_slice(&3u32.to_le_bytes());
        expected.extend_from_slice(b"msg");
        expected.extend_from_slice(&1_700_000_000_i64.to_le_bytes());
        expected.extend_from_slice(&3600i16.to_le_bytes());
        expected.push(5);
        expected.extend_from_slice(b"UTF-8");
        assert_eq!(encoded, expected);
    }

    #[test]
    fn test_encode_commit_with_parents() {
        let commit = Commit::with_meta(
            make_hash(0x01),
            vec![make_hash(0x02), make_hash(0x03)],
            make_user_id("A", "a@b.c"),
            make_user_id("B", "b@c.d"),
            "merge".into(),
            make_meta(0, 0, None),
        )
        .unwrap();
        let mut buf = Cursor::new(Vec::new());
        BinaryEncoder.encode_commit(&commit, &mut buf).unwrap();
        let encoded = buf.into_inner();

        let mut expected = Vec::new();
        expected.push(VERSION);
        expected.extend_from_slice(&hash_bytes(0x01));
        expected.extend_from_slice(&2u16.to_le_bytes());
        expected.extend_from_slice(&hash_bytes(0x02));
        expected.extend_from_slice(&hash_bytes(0x03));
        expected.push(1);
        expected.extend_from_slice(b"A");
        expected.push(5);
        expected.extend_from_slice(b"a@b.c");
        expected.push(1);
        expected.extend_from_slice(b"B");
        expected.push(5);
        expected.extend_from_slice(b"b@c.d");
        expected.extend_from_slice(&5u32.to_le_bytes());
        expected.extend_from_slice(b"merge");
        expected.extend_from_slice(&0i64.to_le_bytes());
        expected.extend_from_slice(&0i16.to_le_bytes());
        expected.push(0);
        assert_eq!(encoded, expected);
    }

    #[test]
    fn test_encode_tag_with_tagger() {
        let tag = Tag::with_meta(
            "v1.0".into(),
            make_hash(0x10),
            Some(make_user_id("Alice", "alice@ex.com")),
            "release".into(),
            make_meta(1_700_000_000, 0, None),
        )
        .unwrap();
        let mut buf = Cursor::new(Vec::new());
        BinaryEncoder.encode_tag(&tag, &mut buf).unwrap();
        let encoded = buf.into_inner();

        let mut expected = Vec::new();
        expected.push(VERSION);
        expected.push(4);
        expected.extend_from_slice(b"v1.0");
        expected.extend_from_slice(&hash_bytes(0x10));
        expected.push(1);
        expected.push(5);
        expected.extend_from_slice(b"Alice");
        expected.push(11);
        expected.extend_from_slice(b"alice@ex.com");
        expected.extend_from_slice(&7u32.to_le_bytes());
        expected.extend_from_slice(b"release");
        expected.extend_from_slice(&1_700_000_000_i64.to_le_bytes());
        expected.extend_from_slice(&0i16.to_le_bytes());
        expected.push(0);
        assert_eq!(encoded, expected);
    }

    #[test]
    fn test_encode_tag_no_tagger() {
        let tag = Tag::with_meta(
            "v2.0".into(),
            make_hash(0x20),
            None,
            "".into(),
            make_meta(1_700_000_000, 0, None),
        )
        .unwrap();
        let mut buf = Cursor::new(Vec::new());
        BinaryEncoder.encode_tag(&tag, &mut buf).unwrap();
        let encoded = buf.into_inner();

        let mut expected = Vec::new();
        expected.push(VERSION);
        expected.push(4);
        expected.extend_from_slice(b"v2.0");
        expected.extend_from_slice(&hash_bytes(0x20));
        expected.push(0);
        expected.extend_from_slice(&0u32.to_le_bytes());
        expected.extend_from_slice(&1_700_000_000_i64.to_le_bytes());
        expected.extend_from_slice(&0i16.to_le_bytes());
        expected.push(0);
        assert_eq!(encoded, expected);
    }

    #[test]
    fn test_encode_tag_with_encoding() {
        let tag = Tag::with_meta(
            "v3".into(),
            make_hash(0x30),
            Some(make_user_id("B", "b@c.d")),
            "tag".into(),
            make_meta(1_700_000_000, -3600, Some("UTF-8")),
        )
        .unwrap();
        let mut buf = Cursor::new(Vec::new());
        BinaryEncoder.encode_tag(&tag, &mut buf).unwrap();
        let encoded = buf.into_inner();

        let mut expected = Vec::new();
        expected.push(VERSION);
        expected.push(2);
        expected.extend_from_slice(b"v3");
        expected.extend_from_slice(&hash_bytes(0x30));
        expected.push(1);
        expected.push(1);
        expected.extend_from_slice(b"B");
        expected.push(5);
        expected.extend_from_slice(b"b@c.d");
        expected.extend_from_slice(&3u32.to_le_bytes());
        expected.extend_from_slice(b"tag");
        expected.extend_from_slice(&1_700_000_000_i64.to_le_bytes());
        expected.extend_from_slice(&(-3600i16).to_le_bytes());
        expected.push(5);
        expected.extend_from_slice(b"UTF-8");
        assert_eq!(encoded, expected);
    }

    #[test]
    fn test_blob_roundtrip() {
        let original_data = vec![0x01, 0x02, 0x03, 0x04, 0x05];
        let blob = Blob::new(original_data.clone()).unwrap();
        let mut buf = Cursor::new(Vec::new());
        BinaryEncoder.encode_blob(&blob, &mut buf).unwrap();
        let encoded = buf.into_inner();

        assert_eq!(encoded[0], VERSION, "first byte should be version");

        let decoded = BinaryDecoder
            .decode_blob(Cursor::new(encoded))
            .expect("decode should succeed");
        assert_eq!(
            decoded.data(),
            original_data.as_slice(),
            "roundtrip blob data should match original"
        );
    }

    #[test]
    fn test_blob_empty_roundtrip() {
        let blob = Blob::new(vec![]).unwrap();
        let mut buf = Cursor::new(Vec::new());
        BinaryEncoder.encode_blob(&blob, &mut buf).unwrap();
        let encoded = buf.into_inner();

        let decoded = BinaryDecoder
            .decode_blob(Cursor::new(encoded))
            .expect("decode empty blob should succeed");
        assert!(
            decoded.data().is_empty(),
            "roundtrip empty blob should have empty data"
        );
    }

    #[test]
    fn test_blob_large_roundtrip() {
        let original_data = vec![0x42u8; 8192];
        let blob = Blob::new(original_data.clone()).unwrap();
        let mut buf = Cursor::new(Vec::new());
        BinaryEncoder.encode_blob(&blob, &mut buf).unwrap();
        let encoded = buf.into_inner();

        let decoded = BinaryDecoder
            .decode_blob(Cursor::new(encoded))
            .expect("decode large blob should succeed");
        assert_eq!(decoded.data(), original_data.as_slice());
    }

    #[test]
    fn test_tree_roundtrip() {
        let e1 = TreeEntry::new("file.txt".into(), EntryKind::Blob, make_hash(0x01)).unwrap();
        let e2 = TreeEntry::new("src".into(), EntryKind::Tree, make_hash(0x02)).unwrap();
        let tree = Tree::new(vec![e1, e2]).unwrap();

        let mut buf = Cursor::new(Vec::new());
        BinaryEncoder.encode_tree(&tree, &mut buf).unwrap();
        let encoded = buf.into_inner();

        let decoded = BinaryDecoder
            .decode_tree(Cursor::new(encoded))
            .expect("decode tree should succeed");
        assert_eq!(decoded.entries().len(), 2);
        assert_eq!(decoded.entries()[0].name(), "file.txt");
        assert_eq!(decoded.entries()[0].kind(), EntryKind::Blob);
        assert_eq!(decoded.entries()[1].name(), "src");
        assert_eq!(decoded.entries()[1].kind(), EntryKind::Tree);
    }

    #[test]
    fn test_commit_roundtrip() {
        let commit = Commit::with_meta(
            make_hash(0x01),
            vec![make_hash(0x02)],
            make_user_id("Alice", "alice@ex.com"),
            make_user_id("Bob", "bob@ex.com"),
            "roundtrip test".into(),
            make_meta(1_700_000_000, 3600, Some("UTF-8")),
        )
        .unwrap();

        let mut buf = Cursor::new(Vec::new());
        BinaryEncoder.encode_commit(&commit, &mut buf).unwrap();
        let encoded = buf.into_inner();

        let decoded = BinaryDecoder
            .decode_commit(Cursor::new(encoded))
            .expect("decode commit should succeed");
        assert_eq!(decoded.parents().len(), 1);
        assert_eq!(decoded.author().name(), "Alice");
        assert_eq!(decoded.committer().name(), "Bob");
        assert_eq!(decoded.message(), "roundtrip test");
        assert_eq!(decoded.meta().timestamp(), 1_700_000_000);
        assert_eq!(decoded.meta().timezone_offset(), 3600);
        assert_eq!(decoded.meta().encoding(), Some("UTF-8"));
    }

    #[test]
    fn test_tag_roundtrip() {
        let tag = Tag::with_meta(
            "v1.0".into(),
            make_hash(0x10),
            Some(make_user_id("Alice", "alice@ex.com")),
            "release tag".into(),
            make_meta(1_700_000_000, 0, None),
        )
        .unwrap();

        let mut buf = Cursor::new(Vec::new());
        BinaryEncoder.encode_tag(&tag, &mut buf).unwrap();
        let encoded = buf.into_inner();

        let decoded = BinaryDecoder
            .decode_tag(Cursor::new(encoded))
            .expect("decode tag should succeed");
        assert_eq!(decoded.name(), "v1.0");
        assert!(decoded.tagger().is_some());
        assert_eq!(decoded.tagger().unwrap().name(), "Alice");
        assert_eq!(decoded.message(), "release tag");
        assert!(decoded.meta().encoding().is_none());
    }

    #[test]
    fn test_tag_roundtrip_no_tagger() {
        let tag = Tag::with_meta(
            "v2.0".into(),
            make_hash(0x20),
            None,
            "".into(),
            make_meta(1_700_000_000, 0, None),
        )
        .unwrap();

        let mut buf = Cursor::new(Vec::new());
        BinaryEncoder.encode_tag(&tag, &mut buf).unwrap();
        let encoded = buf.into_inner();

        let decoded = BinaryDecoder
            .decode_tag(Cursor::new(encoded))
            .expect("decode lightweight tag should succeed");
        assert_eq!(decoded.name(), "v2.0");
        assert!(decoded.tagger().is_none());
        assert_eq!(decoded.message(), "");
    }
}
