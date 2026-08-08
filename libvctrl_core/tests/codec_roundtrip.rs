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
    fn tree_mixed_kinds() {
        let entries = vec![
            TreeEntry::new("blob".into(), EntryKind::Blob, hash_from_byte(1)).unwrap(),
            TreeEntry::new("tree".into(), EntryKind::Tree, hash_from_byte(2)).unwrap(),
        ];
        let t = Tree::new(entries).unwrap();
        let enc = BinaryEncoder.encode_tree(&t).unwrap();
        let dec = BinaryDecoder.decode_tree(&enc).unwrap();
        assert_eq!(dec.entries()[0].kind(), EntryKind::Blob);
        assert_eq!(dec.entries()[1].kind(), EntryKind::Tree);
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
}

mod error_handling {
    use super::*;

    fn valid_blob_encoding(data_len: usize) -> Vec<u8> {
        let blob = Blob::new(vec![0; data_len]);
        BinaryEncoder.encode_blob(&blob).unwrap()
    }

    fn valid_empty_tree_encoding() -> Vec<u8> {
        let tree = Tree::new(vec![]).unwrap();
        BinaryEncoder.encode_tree(&tree).unwrap()
    }

    fn valid_commit_encoding() -> Vec<u8> {
        BinaryEncoder.encode_commit(&minimal_commit()).unwrap()
    }

    fn valid_lightweight_tag_encoding() -> Vec<u8> {
        let tag = lightweight_tag("test");
        BinaryEncoder.encode_tag(&tag).unwrap()
    }

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
        let mut enc = valid_blob_encoding(5);
        enc.push(0x00);
        assert!(BinaryDecoder.decode_blob(&enc).is_err());
    }

    #[test]
    fn blob_length_exceeds_max() {
        let over_size = libvctrl_handler::MAX_BLOB_SIZE + 1;
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
        let over = libvctrl_handler::MAX_TREE_ENTRIES + 1;
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
        let mut enc = valid_empty_tree_encoding();
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
        let mut enc = valid_commit_encoding();
        enc.truncate(10);
        assert!(BinaryDecoder.decode_commit(&enc).is_err());
    }

    #[test]
    fn commit_missing_author_name() {
        let mut enc = valid_commit_encoding();
        enc.truncate(66);
        assert!(BinaryDecoder.decode_commit(&enc).is_err());
    }

    #[test]
    fn commit_author_name_length_mismatch() {
        let mut enc = valid_commit_encoding();
        if enc.len() > 66 {
            enc[66] = 200;
        }
        assert!(BinaryDecoder.decode_commit(&enc).is_err());
    }

    #[test]
    fn commit_message_truncated() {
        let mut enc = valid_commit_encoding();
        enc.truncate(enc.len() - 2);
        assert!(BinaryDecoder.decode_commit(&enc).is_err());
    }

    #[test]
    fn commit_message_too_long() {
        let user = UserID::new("a".into(), "a@b".into()).unwrap();
        let msg = "A".repeat(libvctrl_handler::MAX_MESSAGE_LENGTH + 1);
        let c = Commit::new(dummy_hash(), vec![], user.clone(), user, msg);
        assert!(BinaryEncoder.encode_commit(&c).is_err());
    }

    #[test]
    fn tag_empty_input() {
        assert!(BinaryDecoder.decode_tag(&[]).is_err());
    }

    #[test]
    fn tag_missing_name() {
        let mut enc = valid_lightweight_tag_encoding();
        enc.truncate(1);
        assert!(BinaryDecoder.decode_tag(&enc).is_err());
    }

    #[test]
    fn tag_name_length_mismatch() {
        let mut enc = valid_lightweight_tag_encoding();
        enc[1] = 100;
        assert!(BinaryDecoder.decode_tag(&enc).is_err());
    }

    #[test]
    fn tag_missing_target_hash() {
        let mut enc = valid_lightweight_tag_encoding();
        let name_len = enc[1] as usize;
        let hash_start = 1 + 1 + name_len;
        enc.truncate(hash_start);
        assert!(BinaryDecoder.decode_tag(&enc).is_err());
    }

    #[test]
    fn tag_invalid_tagger_presence_byte() {
        let mut enc = valid_lightweight_tag_encoding();
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
}
