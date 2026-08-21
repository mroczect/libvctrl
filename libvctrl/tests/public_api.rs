use core::str::FromStr;

use libvctrl_core as _;
use libvctrl_handler as _;
use libvctrl_sha512 as _;
use proptest as _;

use libvctrl::{
    BinaryDecoder, BinaryEncoder, Blob, BlobBuilder, Commit, CommitBuilder, CommitMeta, Decoder,
    Encoder, EntryKind, HASH_LENGTH, Hash, Hasher, MemoryRefStore, MemoryStore, ObjectStore,
    RefStore, Sha512Hasher, Tag, TagBuilder, Tree, TreeBuilder, TreeEntry, TreeEntryBuilder,
    UserID, VctrlError, validate_name, validate_ref_name, validate_tree_entry_name,
};

const fn make_hash(byte: u8) -> Result<Hash, VctrlError> {
    Hash::from_bytes(&[byte; 64])
}

fn make_user(name: &str, email: &str) -> Result<UserID, VctrlError> {
    UserID::new(name.to_string(), email.to_string())
}

#[test]
fn hash_roundtrip_through_public_api() -> Result<(), VctrlError> {
    let hash = make_hash(0x42)?;
    assert_eq!(hash.as_bytes().len(), HASH_LENGTH);
    assert_eq!(Hash::from_str(&hash.to_string())?, hash);
    Ok(())
}

#[test]
fn validation_functions_work() -> Result<(), VctrlError> {
    validate_name("valid-name")?;
    assert!(validate_name("").is_err());

    validate_ref_name("refs/heads/main")?;
    assert!(validate_ref_name("refs/heads/.hidden").is_err());
    assert!(validate_ref_name("refs/heads/foo.lock/bar").is_err());
    assert!(validate_ref_name("@").is_err());

    validate_tree_entry_name("file.txt")?;
    assert!(validate_tree_entry_name("dir/file.txt").is_err());

    Ok(())
}

#[test]
fn tree_builder_and_entry_builder_work() -> Result<(), VctrlError> {
    let hash = make_hash(0x11)?;
    let entry = TreeEntryBuilder::new("file.txt".to_string(), EntryKind::Blob, hash).build()?;
    let tree = TreeBuilder::new().entry(entry).build()?;

    let entries = tree.entries();
    assert_eq!(entries.len(), 1);
    let first = entries
        .first()
        .ok_or_else(|| VctrlError::Other("expected entry".into()))?;
    assert_eq!(first.name(), "file.txt");
    assert_eq!(first.kind(), EntryKind::Blob);
    assert_eq!(*first.hash(), hash);
    Ok(())
}

#[test]
fn blob_builder_works() -> Result<(), VctrlError> {
    let data = vec![1_u8, 2, 3, 4];
    let blob = BlobBuilder::new().with_data(data.clone()).build()?;
    assert_eq!(blob.data(), data.as_slice());
    Ok(())
}

#[test]
fn commit_and_tag_builders_work() -> Result<(), VctrlError> {
    let tree_hash = make_hash(0x22)?;
    let parent_hash = make_hash(0x23)?;
    let author = make_user("Alice", "alice@example.com")?;
    let committer = make_user("Bob", "bob@example.com")?;
    let meta = CommitMeta::new(1_600_000_000, 0, Some("utf-8".into()))?;

    let commit = CommitBuilder::new()
        .tree(tree_hash)
        .parent(parent_hash)
        .author(author)
        .committer(committer)
        .message("builder commit")
        .meta(meta)
        .build()?;

    assert_eq!(commit.tree(), &tree_hash);
    assert_eq!(commit.parents(), &[parent_hash]);
    assert_eq!(commit.author().name(), "Alice");
    assert_eq!(commit.committer().email(), "bob@example.com");
    assert_eq!(commit.message(), "builder commit");
    assert_eq!(commit.meta().timestamp(), 1_600_000_000);
    assert_eq!(commit.meta().encoding(), Some("utf-8"));

    let tagger = make_user("Tagger", "tagger@example.com")?;
    let tag = TagBuilder::new()
        .name("v1.0")
        .target(tree_hash)
        .tagger(tagger)
        .message("release")
        .build()?;

    assert_eq!(tag.name(), "v1.0");
    assert_eq!(tag.target(), &tree_hash);
    assert_eq!(
        tag.tagger()
            .ok_or_else(|| VctrlError::Other("expected tagger".into()))?
            .name(),
        "Tagger"
    );
    assert_eq!(tag.message(), "release");
    Ok(())
}

#[test]
fn codec_roundtrip_through_public_api() -> Result<(), VctrlError> {
    let encoder = BinaryEncoder;
    let decoder = BinaryDecoder;

    // Blob
    let blob = Blob::new(b"roundtrip".to_vec())?;
    let mut buf = Vec::new();
    encoder.encode_blob(&blob, &mut buf)?;
    let decoded_blob = decoder.decode_blob(std::io::Cursor::new(buf))?;
    assert_eq!(decoded_blob.data(), blob.data());

    // Tree
    let hash = make_hash(0x33)?;
    let entry = TreeEntry::new("a.txt".to_string(), EntryKind::Blob, hash)?;
    let tree = Tree::new(vec![entry])?;
    let mut buf = Vec::new();
    encoder.encode_tree(&tree, &mut buf)?;
    let decoded_tree = decoder.decode_tree(std::io::Cursor::new(buf))?;
    assert_eq!(decoded_tree.entries().len(), 1);

    // Commit
    let author = make_user("Alice", "alice@example.com")?;
    let committer = make_user("Bob", "bob@example.com")?;
    let meta = CommitMeta::new(1_600_000_000, 0, None)?;
    let commit = Commit::with_meta(
        hash,
        vec![hash],
        author,
        committer,
        "commit".to_string(),
        meta,
    )?;
    let mut buf = Vec::new();
    encoder.encode_commit(&commit, &mut buf)?;
    let decoded_commit = decoder.decode_commit(std::io::Cursor::new(buf))?;
    assert_eq!(decoded_commit.message(), "commit");

    // Tag
    let tagger = make_user("Tagger", "tagger@example.com")?;
    let tag = Tag::with_meta(
        "v1.0".to_string(),
        hash,
        Some(tagger),
        "tag".to_string(),
        CommitMeta::new(1_600_000_000, 0, None)?,
    )?;
    let mut buf = Vec::new();
    encoder.encode_tag(&tag, &mut buf)?;
    let decoded_tag = decoder.decode_tag(std::io::Cursor::new(buf))?;
    assert_eq!(decoded_tag.name(), "v1.0");

    Ok(())
}

#[test]
fn hasher_public_api_works() -> Result<(), VctrlError> {
    let hasher = Sha512Hasher;
    let hash = hasher.hash(std::io::Cursor::new(b"test"))?;
    assert_eq!(hash.as_bytes().len(), 64);
    Ok(())
}

#[test]
fn memory_store_works() -> Result<(), VctrlError> {
    let mut store = MemoryStore::new();
    let hash = make_hash(0x44)?;
    let data = vec![7_u8, 8, 9];

    store.put(&hash, &data)?;
    assert!(store.exists(&hash)?);

    {
        let mut reader = store.get(&hash)?;
        let mut buf = Vec::new();
        let _ = std::io::Read::read_to_end(&mut reader, &mut buf)?;
        assert_eq!(buf, data);
    }

    store.delete(&hash)?;
    assert!(!store.exists(&hash)?);
    Ok(())
}

#[test]
fn memory_ref_store_works() -> Result<(), VctrlError> {
    let mut store = MemoryRefStore::new();
    let hash = make_hash(0x55)?;

    store.set_ref("refs/heads/main", &hash)?;
    assert_eq!(store.get_ref("refs/heads/main")?, hash);

    let refs: Vec<String> = store.list_refs()?.collect::<Result<_, _>>()?;
    assert_eq!(refs, vec!["refs/heads/main".to_string()]);

    store.delete_ref("refs/heads/main")?;
    assert!(store.get_ref("refs/heads/main").is_err());
    Ok(())
}
