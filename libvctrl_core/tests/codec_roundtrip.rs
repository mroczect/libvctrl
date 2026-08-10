use libvctrl_core::codec::{BinaryDecoder, BinaryEncoder};
use libvctrl_handler::{
    Blob, Commit, Decoder, Encoder, EntryKind, Hash, Tag, Tree, TreeEntry, UserID,
};

fn dummy_hash() -> Hash {
    Hash::from_bytes(&[0xAB; 64]).unwrap()
}

fn hash_from_byte(b: u8) -> Hash {
    Hash::from_bytes(&[b; 64]).unwrap()
}

fn blob_of_size(size: usize) -> Blob {
    Blob::new(vec![0x42; size])
}

fn tree_with_n_entries(n: usize) -> Tree {
    let mut entries = Vec::with_capacity(n);
    for i in 0..n {
        let name = format!("entry_{i:03}");
        entries.push(TreeEntry::new(name, EntryKind::Blob, dummy_hash()).unwrap());
    }
    Tree::new(entries).unwrap()
}

fn minimal_commit() -> Commit {
    let user = UserID::new("author".into(), "author@example.com".into()).unwrap();
    Commit::new(dummy_hash(), vec![], user.clone(), user, "message".into())
}

fn lightweight_tag(name: &str) -> Tag {
    Tag::new(name.into(), dummy_hash(), None, String::new()).unwrap()
}

mod roundtrip_success {
    use super::*;

    #[test]
    fn blob_empty() {
        let b = Blob::new(vec![]);
        let enc = BinaryEncoder.encode_blob(&b).unwrap();
        let dec = BinaryDecoder.decode_blob(&enc).unwrap();
        assert_eq!(dec.data(), b"");
    }

    #[test]
    fn blob_small() {
        let b = Blob::new(b"hello world".to_vec());
        let enc = BinaryEncoder.encode_blob(&b).unwrap();
        let dec = BinaryDecoder.decode_blob(&enc).unwrap();
        assert_eq!(dec.data(), b.data());
    }

    #[test]
    fn blob_max_size() {
        let b = blob_of_size(1024);
        let enc = BinaryEncoder.encode_blob(&b).unwrap();
        let dec = BinaryDecoder.decode_blob(&enc).unwrap();
        assert_eq!(dec.data(), b.data());
    }

    #[test]
    fn tree_empty() {
        let t = Tree::new(vec![]).unwrap();
        let enc = BinaryEncoder.encode_tree(&t).unwrap();
        let dec = BinaryDecoder.decode_tree(&enc).unwrap();
        assert!(dec.entries().is_empty());
    }

    #[test]
    fn tree_one_entry() {
        let t = tree_with_n_entries(1);
        let enc = BinaryEncoder.encode_tree(&t).unwrap();
        let dec = BinaryDecoder.decode_tree(&enc).unwrap();
        assert_eq!(dec.entries().len(), 1);
        assert_eq!(dec.entries()[0].name(), "entry_000");
    }

    #[test]
    fn tree_multiple_entries() {
        let t = tree_with_n_entries(5);
        let enc = BinaryEncoder.encode_tree(&t).unwrap();
        let dec = BinaryDecoder.decode_tree(&enc).unwrap();
        assert_eq!(dec.entries().len(), 5);
        for (a, b) in t.entries().iter().zip(dec.entries().iter()) {
            assert_eq!(a.name(), b.name());
            assert_eq!(a.kind(), b.kind());
            assert_eq!(a.hash(), b.hash());
        }
    }

    #[test]
    fn tree_all_entry_kinds() {
        let entries = vec![
            TreeEntry::new("blob".into(), EntryKind::Blob, hash_from_byte(1)).unwrap(),
            TreeEntry::new("dir".into(), EntryKind::Tree, hash_from_byte(4)).unwrap(),
            TreeEntry::new("exec".into(), EntryKind::Executable, hash_from_byte(2)).unwrap(),
            TreeEntry::new("link".into(), EntryKind::Symlink, hash_from_byte(3)).unwrap(),
            TreeEntry::new("sub".into(), EntryKind::Submodule, hash_from_byte(5)).unwrap(),
        ];
        let t = Tree::new(entries).unwrap();
        let enc = BinaryEncoder.encode_tree(&t).unwrap();
        let dec = BinaryDecoder.decode_tree(&enc).unwrap();
        assert_eq!(dec.entries().len(), 5);
        assert_eq!(dec.entries()[0].kind(), EntryKind::Blob);
        assert_eq!(dec.entries()[1].kind(), EntryKind::Tree);
        assert_eq!(dec.entries()[2].kind(), EntryKind::Executable);
        assert_eq!(dec.entries()[3].kind(), EntryKind::Symlink);
        assert_eq!(dec.entries()[4].kind(), EntryKind::Submodule);
    }

    #[test]
    fn commit_no_parents() {
        let c = minimal_commit();
        let enc = BinaryEncoder.encode_commit(&c).unwrap();
        let dec = BinaryDecoder.decode_commit(&enc).unwrap();
        assert_eq!(dec.tree(), c.tree());
        assert!(dec.parents().is_empty());
        assert_eq!(dec.author().name(), "author");
        assert_eq!(dec.committer().name(), "author");
        assert_eq!(dec.message(), "message");
        assert_eq!(dec.timestamp(), 0);
        assert_eq!(dec.timezone_offset(), 0);
        assert!(dec.encoding().is_none());
    }

    #[test]
    fn commit_with_parents() {
        let user = UserID::new("bob".into(), "bob@example.com".into()).unwrap();
        let parents = vec![hash_from_byte(1), hash_from_byte(2), hash_from_byte(3)];
        let c = Commit::new(dummy_hash(), parents, user.clone(), user, "merge".into());
        let enc = BinaryEncoder.encode_commit(&c).unwrap();
        let dec = BinaryDecoder.decode_commit(&enc).unwrap();
        assert_eq!(dec.parents().len(), 3);
        assert_eq!(dec.parents()[1], hash_from_byte(2));
    }

    #[test]
    fn commit_different_author_committer() {
        let author = UserID::new("author".into(), "a@b.c".into()).unwrap();
        let committer = UserID::new("committer".into(), "c@d.e".into()).unwrap();
        let c = Commit::new(dummy_hash(), vec![], author, committer, "msg".into());
        let enc = BinaryEncoder.encode_commit(&c).unwrap();
        let dec = BinaryDecoder.decode_commit(&enc).unwrap();
        assert_ne!(dec.author().name(), dec.committer().name());
    }

    #[test]
    fn commit_with_encoding() {
        let user = UserID::new("a".into(), "a@b".into()).unwrap();
        let meta = libvctrl_handler::CommitMeta {
            timestamp: 1,
            timezone_offset: 2,
            encoding: Some("UTF-8".into()),
        };
        let c = Commit::with_meta(dummy_hash(), vec![], user.clone(), user, "msg".into(), meta);
        let enc = BinaryEncoder.encode_commit(&c).unwrap();
        let dec = BinaryDecoder.decode_commit(&enc).unwrap();
        assert_eq!(dec.encoding(), Some("UTF-8"));
        assert_eq!(dec.timestamp(), 1);
        assert_eq!(dec.timezone_offset(), 2);
    }

    #[test]
    fn tag_lightweight() {
        let t = lightweight_tag("v0.1");
        let enc = BinaryEncoder.encode_tag(&t).unwrap();
        let dec = BinaryDecoder.decode_tag(&enc).unwrap();
        assert_eq!(dec.name(), "v0.1");
        assert!(dec.tagger().is_none());
        assert!(dec.message().is_empty());
        assert_eq!(dec.timestamp(), 0);
        assert_eq!(dec.timezone_offset(), 0);
        assert!(dec.encoding().is_none());
    }

    #[test]
    fn tag_with_tagger_and_message() {
        let tagger = UserID::new("tagger".into(), "tag@example.com".into()).unwrap();
        let t = Tag::new(
            "v1.0".into(),
            dummy_hash(),
            Some(tagger),
            "Release notes".into(),
        )
        .unwrap();
        let enc = BinaryEncoder.encode_tag(&t).unwrap();
        let dec = BinaryDecoder.decode_tag(&enc).unwrap();
        assert_eq!(dec.tagger().unwrap().name(), "tagger");
        assert_eq!(dec.message(), "Release notes");
    }

    #[test]
    fn tag_with_encoding() {
        let tagger = UserID::new("t".into(), "t@t".into()).unwrap();
        let meta = libvctrl_handler::CommitMeta {
            timestamp: 3,
            timezone_offset: 4,
            encoding: Some("ISO-8859-1".into()),
        };
        let t = Tag::with_meta(
            "v2.0".into(),
            dummy_hash(),
            Some(tagger),
            "msg".into(),
            meta,
        )
        .unwrap();
        let enc = BinaryEncoder.encode_tag(&t).unwrap();
        let dec = BinaryDecoder.decode_tag(&enc).unwrap();
        assert_eq!(dec.encoding(), Some("ISO-8859-1"));
        assert_eq!(dec.timestamp(), 3);
        assert_eq!(dec.timezone_offset(), 4);
    }
}

mod error_handling {
    use super::*;

    #[test]
    fn blob_empty_input() {
        assert!(BinaryDecoder.decode_blob(&[]).is_err());
    }

    #[test]
    fn blob_missing_length() {
        let mut data = vec![0x02];
        assert!(BinaryDecoder.decode_blob(&data).is_err());
        data.extend_from_slice(&[0; 4]);
        assert!(BinaryDecoder.decode_blob(&data).is_err());
    }

    #[test]
    fn blob_length_mismatch() {
        let blob = Blob::new(vec![0; 5]);
        let mut enc = BinaryEncoder.encode_blob(&blob).unwrap();
        enc.push(0x00);
        assert!(BinaryDecoder.decode_blob(&enc).is_err());
    }

    #[test]
    fn blob_length_exceeds_max() {
        let max_blob_size =
            usize::try_from(libvctrl_handler::MAX_BLOB_SIZE).expect("MAX_BLOB_SIZE too large");
        let over_size = max_blob_size + 1;
        let mut bytes = vec![0x02u8];
        bytes.extend_from_slice(&(over_size as u64).to_le_bytes());
        bytes.extend(vec![0x00; over_size]);
        assert!(BinaryDecoder.decode_blob(&bytes).is_err());
    }

    #[test]
    fn tree_empty_input() {
        assert!(BinaryDecoder.decode_tree(&[]).is_err());
    }

    #[test]
    fn tree_missing_entry_count() {
        let mut enc = vec![0x02];
        assert!(BinaryDecoder.decode_tree(&enc).is_err());
        enc.push(0x00);
        assert!(BinaryDecoder.decode_tree(&enc).is_err());
    }

    #[test]
    fn tree_entry_count_exceeds_max() {
        let over = usize::try_from(libvctrl_handler::MAX_TREE_ENTRIES)
            .expect("MAX_TREE_ENTRIES too large")
            + 1;
        let mut enc = vec![0x02u8];
        enc.extend_from_slice(
            &u32::try_from(over)
                .expect("MAX_TREE_ENTRIES is small enough to fit in u32")
                .to_le_bytes(),
        );
        assert!(BinaryDecoder.decode_tree(&enc).is_err());
    }

    #[test]
    fn tree_truncated_entry_name() {
        let tree = Tree::new(vec![]).unwrap();
        let mut enc = BinaryEncoder.encode_tree(&tree).unwrap();
        enc[1..5].copy_from_slice(&1u32.to_le_bytes());
        enc.push(50);
        assert!(BinaryDecoder.decode_tree(&enc).is_err());
    }

    #[test]
    fn tree_invalid_entry_kind() {
        let tree = tree_with_n_entries(1);
        let mut enc = BinaryEncoder.encode_tree(&tree).unwrap();
        let kind_pos = 6 + 9;
        enc[kind_pos] = 99;
        assert!(BinaryDecoder.decode_tree(&enc).is_err());
    }

    #[test]
    fn tree_truncated_hash() {
        let tree = tree_with_n_entries(1);
        let mut enc = BinaryEncoder.encode_tree(&tree).unwrap();
        enc.truncate(enc.len() - 4);
        assert!(BinaryDecoder.decode_tree(&enc).is_err());
    }

    #[test]
    fn commit_empty_input() {
        assert!(BinaryDecoder.decode_commit(&[]).is_err());
    }

    #[test]
    fn commit_too_short() {
        let mut enc = BinaryEncoder.encode_commit(&minimal_commit()).unwrap();
        enc.truncate(10);
        assert!(BinaryDecoder.decode_commit(&enc).is_err());
    }

    #[test]
    fn commit_missing_author_name() {
        let mut enc = BinaryEncoder.encode_commit(&minimal_commit()).unwrap();
        enc.truncate(66);
        assert!(BinaryDecoder.decode_commit(&enc).is_err());
    }

    #[test]
    fn commit_author_name_length_mismatch() {
        let mut enc = BinaryEncoder.encode_commit(&minimal_commit()).unwrap();
        if enc.len() > 66 {
            enc[66] = 200;
        }
        assert!(BinaryDecoder.decode_commit(&enc).is_err());
    }

    #[test]
    fn commit_message_truncated() {
        let mut enc = BinaryEncoder.encode_commit(&minimal_commit()).unwrap();
        enc.truncate(enc.len() - 2);
        assert!(BinaryDecoder.decode_commit(&enc).is_err());
    }

    #[test]
    fn commit_message_too_long() {
        let user = UserID::new("a".into(), "a@b".into()).unwrap();
        let msg_len = usize::try_from(libvctrl_handler::MAX_MESSAGE_LENGTH)
            .expect("MAX_MESSAGE_LENGTH too large")
            + 1;
        let msg = "A".repeat(msg_len);
        let c = Commit::new(dummy_hash(), vec![], user.clone(), user, msg);
        assert!(BinaryEncoder.encode_commit(&c).is_err());
    }

    #[test]
    fn tag_empty_input() {
        assert!(BinaryDecoder.decode_tag(&[]).is_err());
    }

    #[test]
    fn tag_missing_name() {
        let mut enc = BinaryEncoder.encode_tag(&lightweight_tag("test")).unwrap();
        enc.truncate(1);
        assert!(BinaryDecoder.decode_tag(&enc).is_err());
    }

    #[test]
    fn tag_name_length_mismatch() {
        let mut enc = BinaryEncoder.encode_tag(&lightweight_tag("test")).unwrap();
        enc[1] = 100;
        assert!(BinaryDecoder.decode_tag(&enc).is_err());
    }

    #[test]
    fn tag_missing_target_hash() {
        let mut enc = BinaryEncoder.encode_tag(&lightweight_tag("test")).unwrap();
        let name_len = enc[1] as usize;
        let hash_start = 1 + 1 + name_len;
        enc.truncate(hash_start);
        assert!(BinaryDecoder.decode_tag(&enc).is_err());
    }

    #[test]
    fn tag_invalid_tagger_presence_byte() {
        let mut enc = BinaryEncoder.encode_tag(&lightweight_tag("test")).unwrap();
        let name_len = enc[1] as usize;
        let presence_pos = 1 + 1 + name_len + 64;
        enc[presence_pos] = 2;
        assert!(BinaryDecoder.decode_tag(&enc).is_err());
    }

    #[test]
    fn tag_message_truncated() {
        let tagger = UserID::new("t".into(), "t@t".into()).unwrap();
        let tag = Tag::new(
            "v".into(),
            dummy_hash(),
            Some(tagger),
            "some message".into(),
        )
        .unwrap();
        let mut enc = BinaryEncoder.encode_tag(&tag).unwrap();
        enc.truncate(enc.len() - 1);
        assert!(BinaryDecoder.decode_tag(&enc).is_err());
    }

    #[test]
    fn tag_message_too_long() {
        let tagger = UserID::new("t".into(), "t@t".into()).unwrap();
        let msg_len = usize::try_from(libvctrl_handler::MAX_MESSAGE_LENGTH)
            .expect("MAX_MESSAGE_LENGTH too large")
            + 1;
        let msg = "A".repeat(msg_len);
        let tag = Tag::new("v".into(), dummy_hash(), Some(tagger), msg).unwrap();
        assert!(BinaryEncoder.encode_tag(&tag).is_err());
    }
}

mod version_handling {
    use super::*;

    #[test]
    fn correct_version_accepted() {
        let blob = blob_of_size(0);
        let enc = BinaryEncoder.encode_blob(&blob).unwrap();
        assert_eq!(enc[0], 2);
        assert!(BinaryDecoder.decode_blob(&enc).is_ok());
    }

    #[test]
    fn wrong_version_rejected() {
        let mut enc = vec![0x01u8];
        enc.extend_from_slice(&0u64.to_le_bytes());
        assert!(BinaryDecoder.decode_blob(&enc).is_err());

        let mut enc2 = vec![0x00u8];
        enc2.extend_from_slice(&0u64.to_le_bytes());
        assert!(BinaryDecoder.decode_blob(&enc2).is_err());
    }

    #[test]
    fn missing_version_byte() {
        assert!(BinaryDecoder.decode_blob(&[]).is_err());
    }
}
