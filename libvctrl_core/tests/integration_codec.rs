use std::io::Cursor;

use libvctrl_sha512 as _;
use proptest as _;

use libvctrl_core::codec::{BinaryDecoder, BinaryEncoder};
use libvctrl_handler::{
    Blob, Commit, CommitMeta, Decoder, Encoder, EntryKind, Tag, Tree, TreeEntry, UserID, VctrlError,
};

pub mod common;

fn make_user(name: &str, email: &str) -> Result<UserID, VctrlError> {
    UserID::new(name.to_string(), email.to_string())
}

fn make_meta(ts: i64, tz: i16) -> Result<CommitMeta, VctrlError> {
    CommitMeta::new(ts, tz, None)
}

#[test]
fn blob_roundtrip_through_public_api() -> Result<(), VctrlError> {
    let payload = vec![9_u8, 8, 7, 6];
    let blob = Blob::new(payload.clone())?;

    let mut buf = Vec::new();
    BinaryEncoder.encode_blob(&blob, &mut buf)?;

    let decoded = BinaryDecoder.decode_blob(Cursor::new(buf))?;
    assert_eq!(decoded.data(), payload.as_slice());

    Ok(())
}

#[test]
fn tree_roundtrip_through_public_api() -> Result<(), VctrlError> {
    let hash = common::make_hash(0x44)?;
    let entry = TreeEntry::new("file.txt".to_string(), EntryKind::Executable, hash)?;
    let tree = Tree::new(vec![entry])?;

    let mut buf = Vec::new();
    BinaryEncoder.encode_tree(&tree, &mut buf)?;

    let decoded = BinaryDecoder.decode_tree(Cursor::new(buf))?;
    assert_eq!(decoded.entries().len(), 1);
    let first = decoded
        .entries()
        .first()
        .ok_or_else(|| VctrlError::Other("expected entry".into()))?;
    assert_eq!(first.name(), "file.txt");
    assert_eq!(first.kind(), EntryKind::Executable);
    assert_eq!(*first.hash(), hash);

    Ok(())
}

#[test]
fn commit_roundtrip_through_public_api() -> Result<(), VctrlError> {
    let tree = common::make_hash(0x55)?;
    let parent = common::make_hash(0x56)?;
    let author = make_user("Alice", "alice@example.com")?;
    let committer = make_user("Bob", "bob@example.com")?;
    let message = "integration commit".to_string();
    let meta = make_meta(1_600_000_000, 0)?;

    let commit = Commit::with_meta(tree, vec![parent], author, committer, message.clone(), meta)?;

    let mut buf = Vec::new();
    BinaryEncoder.encode_commit(&commit, &mut buf)?;

    let decoded = BinaryDecoder.decode_commit(Cursor::new(buf))?;
    assert_eq!(decoded.tree(), &tree);
    assert_eq!(decoded.parents(), &[parent]);
    assert_eq!(decoded.author().name(), "Alice");
    assert_eq!(decoded.committer().email(), "bob@example.com");
    assert_eq!(decoded.message(), message);
    assert_eq!(decoded.meta().timestamp(), 1_600_000_000);
    assert_eq!(decoded.meta().timezone_offset(), 0);

    Ok(())
}

#[test]
fn tag_roundtrip_through_public_api() -> Result<(), VctrlError> {
    let target = common::make_hash(0x66)?;
    let tagger = make_user("Tagger", "tagger@example.com")?;
    let message = "v1.0".to_string();
    let meta = make_meta(1_600_000_000, 0)?;

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
    let tagger = decoded
        .tagger()
        .ok_or_else(|| VctrlError::Other("expected tagger".into()))?;
    assert_eq!(tagger.name(), "Tagger");
    assert_eq!(decoded.message(), message);
    assert_eq!(decoded.meta().timestamp(), 1_600_000_000);
    assert_eq!(decoded.meta().timezone_offset(), 0);

    Ok(())
}
