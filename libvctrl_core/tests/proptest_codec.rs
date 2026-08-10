use libvctrl_core::codec::{BinaryDecoder, BinaryEncoder};
use libvctrl_handler::{
    Blob, Commit, Decoder, Encoder, EntryKind, Hash, Tag, Tree, TreeEntry, UserID,
};
use proptest::prelude::*;

fn hash_strategy() -> impl Strategy<Value = Hash> {
    any::<[u8; 64]>().prop_map(|bytes| Hash::from_bytes(&bytes).unwrap())
}

fn name_strategy() -> impl Strategy<Value = String> {
    proptest::string::string_regex("[a-zA-Z][a-zA-Z0-9]{0,30}")
        .unwrap()
        .prop_filter("name must not be '.' or '..'", |s| s != "." && s != "..")
}

fn email_strategy() -> impl Strategy<Value = String> {
    proptest::string::string_regex("[a-z]{1,10}@[a-z]{1,10}\\.[a-z]{2,4}").unwrap()
}

fn user_id_strategy() -> impl Strategy<Value = UserID> {
    (name_strategy(), email_strategy()).prop_map(|(name, email)| UserID::new(name, email).unwrap())
}

fn blob_strategy() -> impl Strategy<Value = Blob> {
    proptest::collection::vec(any::<u8>(), 0..64 * 1024).prop_map(Blob::new)
}

fn entry_kind_strategy() -> impl Strategy<Value = EntryKind> {
    prop_oneof![
        Just(EntryKind::Blob),
        Just(EntryKind::Executable),
        Just(EntryKind::Symlink),
        Just(EntryKind::Tree),
        Just(EntryKind::Submodule),
    ]
}

fn tree_entry_strategy() -> impl Strategy<Value = TreeEntry> {
    (name_strategy(), entry_kind_strategy(), hash_strategy())
        .prop_map(|(name, kind, hash)| TreeEntry::new(name, kind, hash).unwrap())
}

fn tree_strategy() -> impl Strategy<Value = Tree> {
    proptest::collection::vec(tree_entry_strategy(), 0..20).prop_map(|mut entries| {
        entries.sort_by(|a, b| a.name().cmp(b.name()));
        entries.dedup_by(|a, b| a.name() == b.name());
        Tree::new(entries).unwrap()
    })
}

fn message_strategy() -> impl Strategy<Value = String> {
    proptest::string::string_regex("[a-zA-Z0-9 ]{0,1000}").unwrap()
}

fn commit_strategy() -> impl Strategy<Value = Commit> {
    (
        hash_strategy(),
        proptest::collection::vec(hash_strategy(), 0..3),
        user_id_strategy(),
        user_id_strategy(),
        message_strategy(),
    )
        .prop_map(|(tree, parents, author, committer, message)| {
            Commit::new(tree, parents, author, committer, message)
        })
}

fn tag_strategy() -> impl Strategy<Value = Tag> {
    (
        name_strategy(),
        hash_strategy(),
        proptest::option::of(user_id_strategy()),
        message_strategy(),
    )
        .prop_map(|(name, target, tagger, message)| {
            Tag::new(name, target, tagger, message).unwrap()
        })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_roundtrip_blob(blob in blob_strategy()) {
        let encoded = BinaryEncoder.encode_blob(&blob).unwrap();
        let decoded = BinaryDecoder.decode_blob(&encoded).unwrap();
        assert_eq!(decoded.data(), blob.data());
    }

    #[test]
    fn prop_roundtrip_tree(tree in tree_strategy()) {
        let encoded = BinaryEncoder.encode_tree(&tree).unwrap();
        let decoded = BinaryDecoder.decode_tree(&encoded).unwrap();
        assert_eq!(decoded.entries().len(), tree.entries().len());
        for (a, b) in tree.entries().iter().zip(decoded.entries().iter()) {
            assert_eq!(a.name(), b.name());
            assert_eq!(a.kind(), b.kind());
            assert_eq!(a.hash(), b.hash());
        }
    }

    #[test]
    fn prop_roundtrip_commit(commit in commit_strategy()) {
        let encoded = BinaryEncoder.encode_commit(&commit).unwrap();
        let decoded = BinaryDecoder.decode_commit(&encoded).unwrap();
        assert_eq!(commit.tree(), decoded.tree());
        assert_eq!(commit.parents().len(), decoded.parents().len());
        assert_eq!(commit.author().name(), decoded.author().name());
        assert_eq!(commit.committer().name(), decoded.committer().name());
        assert_eq!(commit.message(), decoded.message());
    }

    #[test]
    fn prop_roundtrip_tag(tag in tag_strategy()) {
        let encoded = BinaryEncoder.encode_tag(&tag).unwrap();
        let decoded = BinaryDecoder.decode_tag(&encoded).unwrap();
        assert_eq!(tag.name(), decoded.name());
        assert_eq!(tag.target(), decoded.target());
        assert_eq!(tag.tagger().map(|u| u.name().to_string()), decoded.tagger().map(|u| u.name().to_string()));
        assert_eq!(tag.message(), decoded.message());
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn prop_fuzz_blob(blob in blob_strategy(), truncate_pos in any::<usize>()) {
        let mut encoded = BinaryEncoder.encode_blob(&blob).unwrap();
        if encoded.is_empty() {
            return Ok(());
        }
        let idx = truncate_pos % encoded.len();
        encoded.truncate(idx);
        assert!(BinaryDecoder.decode_blob(&encoded).is_err());
    }

    #[test]
    fn prop_fuzz_tree(tree in tree_strategy(), truncate_pos in any::<usize>()) {
        let mut encoded = BinaryEncoder.encode_tree(&tree).unwrap();
        if encoded.is_empty() {
            return Ok(());
        }
        let idx = truncate_pos % encoded.len();
        encoded.truncate(idx);
        assert!(BinaryDecoder.decode_tree(&encoded).is_err());
    }

    #[test]
    fn prop_fuzz_commit(commit in commit_strategy(), truncate_pos in any::<usize>()) {
        let mut encoded = BinaryEncoder.encode_commit(&commit).unwrap();
        if encoded.is_empty() {
            return Ok(());
        }
        let idx = truncate_pos % encoded.len();
        encoded.truncate(idx);
        assert!(BinaryDecoder.decode_commit(&encoded).is_err());
    }

    #[test]
    fn prop_fuzz_tag(tag in tag_strategy(), truncate_pos in any::<usize>()) {
        let mut encoded = BinaryEncoder.encode_tag(&tag).unwrap();
        if encoded.is_empty() {
            return Ok(());
        }
        let idx = truncate_pos % encoded.len();
        encoded.truncate(idx);
        assert!(BinaryDecoder.decode_tag(&encoded).is_err());
    }
}
