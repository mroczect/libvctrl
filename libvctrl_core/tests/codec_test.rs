//! # Codec Round-Trip and Limit Tests
//!
//! This test module validates the binary encoder and decoder for all core
//! object types: [`Blob`], [`Tree`], [`Commit`], and [`Tag`].
//!
//! The tests verify:
//!
//! - Successful round-trip serialization for valid objects.
//! - Malformed byte streams are rejected with [`VctrlError`].
//! - System limits (`MAX_BLOB_SIZE`, `MAX_TREE_ENTRIES`,
//!   `MAX_PARENT_COUNT`, `MAX_MESSAGE_LENGTH`) are enforced.
//! - Version byte is checked.
//! - All [`EntryKind`] variants survive encoding and decoding.
//!
//! These tests are integration-style but located within the same crate.
//! They help ensure the codec remains backward-compatible and robust against
//! corrupted or malicious input.

use libvctrl_core::codec::{BinaryDecoder, BinaryEncoder};
use libvctrl_handler::{
    Blob, Commit, CommitMeta, Decoder, Encoder, EntryKind, Hash, MAX_BLOB_SIZE, MAX_MESSAGE_LENGTH,
    MAX_PARENT_COUNT, MAX_TREE_ENTRIES, Tag, Tree, TreeEntry, UserID,
};
use std::io::Cursor;

/// Returns a hash filled with the byte `0xAB`.
///
/// This is useful as a placeholder for an arbitrary valid object ID.
fn dummy_hash() -> Hash {
    Hash::from_bytes(&[0xAB; 64]).unwrap()
}

/// Returns a hash filled with the given byte.
///
/// The byte `b` is repeated 64 times to form a valid [`Hash`]. This helper
/// creates distinguishable hashes for testing equality and ordering.
fn hash_from_byte(b: u8) -> Hash {
    Hash::from_bytes(&[b; 64]).unwrap()
}

/// Creates a [`Blob`] of the specified size, filled with `0x42`.
///
/// The size must not exceed [`MAX_BLOB_SIZE`]. The resulting blob is used to
/// test size limits and round-trip behavior.
fn blob_of_size(size: usize) -> Blob {
    Blob::new(vec![0x42; size]).unwrap()
}

/// Creates a [`Tree`] with `n` entries.
///
/// Each entry is named `entry_XXX` (zero-padded) and points to
/// [`dummy_hash`]. The entries are sorted by name to satisfy [`Tree`]
/// ordering requirements.
fn tree_with_n_entries(n: usize) -> Tree {
    let mut entries = Vec::with_capacity(n);
    for i in 0..n {
        let name = format!("entry_{i:03}");
        entries.push(TreeEntry::new(name, EntryKind::Blob, dummy_hash()).unwrap());
    }
    Tree::new(entries).unwrap()
}

/// Creates a minimal, parentless commit with a fixed author and message.
///
/// The tree is [`dummy_hash`], the author and committer are both
/// "author <author@example.com>", and the message is "message".
fn minimal_commit() -> Commit {
    let user = UserID::new("author".into(), "author@example.com".into()).unwrap();
    Commit::new(dummy_hash(), vec![], user.clone(), user, "message".into()).unwrap()
}

/// Creates a lightweight tag (no tagger, empty message) with the given name.
///
/// The target is [`dummy_hash`].
fn lightweight_tag(name: &str) -> Tag {
    Tag::new(name.into(), dummy_hash(), None, String::new()).unwrap()
}

/// Tests blob encoding/decoding and blob size limits.
///
/// Checks:
/// - Empty blob round-trips.
/// - Small blob round-trips.
/// - Blob of exactly `MAX_BLOB_SIZE` round-trips.
/// - Blob exceeding `MAX_BLOB_SIZE` fails at construction.
#[test]
fn test_blob_roundtrip_and_limits() {
    // 1. Empty blob
    let b = Blob::new(vec![]).unwrap();
    let mut enc = Vec::new();
    BinaryEncoder.encode_blob(&b, &mut enc).unwrap();
    let dec = BinaryDecoder.decode_blob(Cursor::new(&enc)).unwrap();
    assert_eq!(dec.data(), b.data());

    // 2. Small blob
    let b = Blob::new(b"hello world".to_vec()).unwrap();
    let mut enc = Vec::new();
    BinaryEncoder.encode_blob(&b, &mut enc).unwrap();
    let dec = BinaryDecoder.decode_blob(Cursor::new(&enc)).unwrap();
    assert_eq!(dec.data(), b.data());

    // 3. Max size blob
    let max_size = usize::try_from(MAX_BLOB_SIZE).unwrap();
    let b = blob_of_size(max_size);
    let mut enc = Vec::new();
    BinaryEncoder.encode_blob(&b, &mut enc).unwrap();
    let dec = BinaryDecoder.decode_blob(Cursor::new(&enc)).unwrap();
    assert_eq!(dec.size(), max_size);

    // 4. Exceeds max size (should fail at Blob::new)
    let over_size = max_size + 1;
    assert!(Blob::new(vec![0; over_size]).is_err());
}

/// Tests that malformed blob inputs are rejected.
///
/// Covers:
/// - Empty input.
/// - Correct version but missing length prefix.
/// - Wrong version byte.
/// - Length mismatch (trailing byte).
/// - Declared length exceeding `MAX_BLOB_SIZE`.
#[test]
fn test_blob_malformed_data() {
    // Empty input
    assert!(BinaryDecoder.decode_blob(Cursor::new(&[])).is_err());

    // Correct version but missing length prefix
    let data = vec![0x03];
    assert!(BinaryDecoder.decode_blob(Cursor::new(&data)).is_err());

    // Wrong version
    let data = vec![0x02];
    assert!(BinaryDecoder.decode_blob(Cursor::new(&data)).is_err());

    // Length mismatch (trailing byte)
    let b = Blob::new(vec![0; 5]).unwrap();
    let mut enc = Vec::new();
    BinaryEncoder.encode_blob(&b, &mut enc).unwrap();
    enc.push(0x00);
    assert!(BinaryDecoder.decode_blob(Cursor::new(&enc)).is_err());

    // Declared length exceeds MAX_BLOB_SIZE
    let over_size = usize::try_from(MAX_BLOB_SIZE).unwrap() + 1;
    let mut bytes = vec![0x03u8];
    bytes.extend_from_slice(&(over_size as u64).to_le_bytes());
    bytes.extend(vec![0x00; over_size]);
    assert!(BinaryDecoder.decode_blob(Cursor::new(&bytes)).is_err());
}

/// Tests tree encoding/decoding and limit enforcement.
///
/// Verifies:
/// - Empty tree round-trips.
/// - Tree with multiple entries round-trips.
/// - All [`EntryKind`] variants survive round-trip.
#[test]
fn test_tree_roundtrip_and_limits() {
    // Empty tree
    let t = Tree::new(vec![]).unwrap();
    let mut enc = Vec::new();
    BinaryEncoder.encode_tree(&t, &mut enc).unwrap();
    let dec = BinaryDecoder.decode_tree(Cursor::new(&enc)).unwrap();
    assert!(dec.entries().is_empty());

    // Multiple entries
    let t = tree_with_n_entries(5);
    let mut enc = Vec::new();
    BinaryEncoder.encode_tree(&t, &mut enc).unwrap();
    let dec = BinaryDecoder.decode_tree(Cursor::new(&enc)).unwrap();
    assert_eq!(dec.entries().len(), 5);

    // All entry kinds roundtrip
    let entries = vec![
        TreeEntry::new("blob".into(), EntryKind::Blob, hash_from_byte(1)).unwrap(),
        TreeEntry::new("dir".into(), EntryKind::Tree, hash_from_byte(4)).unwrap(),
        TreeEntry::new("exec".into(), EntryKind::Executable, hash_from_byte(2)).unwrap(),
        TreeEntry::new("link".into(), EntryKind::Symlink, hash_from_byte(3)).unwrap(),
        TreeEntry::new("sub".into(), EntryKind::Submodule, hash_from_byte(5)).unwrap(),
    ];
    let t = Tree::new(entries).unwrap();
    let mut enc = Vec::new();
    BinaryEncoder.encode_tree(&t, &mut enc).unwrap();
    let dec = BinaryDecoder.decode_tree(Cursor::new(&enc)).unwrap();
    assert_eq!(dec.entries().len(), 5);
}

/// Tests that malformed tree inputs are rejected.
///
/// Covers:
/// - Empty input.
/// - Missing entry count.
/// - Wrong version.
/// - Entry count exceeding `MAX_TREE_ENTRIES`.
/// - Truncated name.
/// - Invalid entry kind byte.
/// - Truncated hash.
/// - Trailing bytes.
#[test]
fn test_tree_malformed_data() {
    // Empty input
    assert!(BinaryDecoder.decode_tree(Cursor::new(&[])).is_err());

    // Correct version but missing entry count bytes
    let data = vec![0x03];
    assert!(BinaryDecoder.decode_tree(Cursor::new(&data)).is_err());

    // Wrong version
    let data = vec![0x02];
    assert!(BinaryDecoder.decode_tree(Cursor::new(&data)).is_err());

    // Entry count exceeds MAX_TREE_ENTRIES
    let over = usize::try_from(MAX_TREE_ENTRIES).unwrap() + 1;
    let mut enc = vec![0x03u8];
    enc.extend_from_slice(&u32::try_from(over).unwrap().to_le_bytes());
    assert!(BinaryDecoder.decode_tree(Cursor::new(&enc)).is_err());

    // Truncated entry name
    let tree = Tree::new(vec![]).unwrap();
    let mut enc = Vec::new();
    BinaryEncoder.encode_tree(&tree, &mut enc).unwrap();
    enc[1..5].copy_from_slice(&1u32.to_le_bytes());
    enc.push(50); // Name length 50, but no data
    assert!(BinaryDecoder.decode_tree(Cursor::new(&enc)).is_err());

    // Invalid entry kind
    let tree = tree_with_n_entries(1);
    let mut enc = Vec::new();
    BinaryEncoder.encode_tree(&tree, &mut enc).unwrap();
    let kind_pos = 6 + 9; // version + count + name_len + name
    enc[kind_pos] = 99;
    assert!(BinaryDecoder.decode_tree(Cursor::new(&enc)).is_err());

    // Truncated hash
    let tree = tree_with_n_entries(1);
    let mut enc = Vec::new();
    BinaryEncoder.encode_tree(&tree, &mut enc).unwrap();
    enc.truncate(enc.len() - 4);
    assert!(BinaryDecoder.decode_tree(Cursor::new(&enc)).is_err());

    // Trailing bytes
    let tree = tree_with_n_entries(1);
    let mut enc = Vec::new();
    BinaryEncoder.encode_tree(&tree, &mut enc).unwrap();
    enc.push(0x00);
    assert!(BinaryDecoder.decode_tree(Cursor::new(&enc)).is_err());
}

/// Tests commit encoding/decoding and limit enforcement.
///
/// Verifies:
/// - Minimal commit round-trips.
/// - Commits with 0–256 parents round-trip.
/// - Duplicate parents are rejected.
/// - Parent count exceeding `MAX_PARENT_COUNT` is rejected.
/// - Metadata encoding survives round-trip.
/// - Invalid timezone offset is rejected.
/// - Message exceeding `MAX_MESSAGE_LENGTH` is rejected.
#[test]
fn test_commit_roundtrip_and_limits() {
    let user = UserID::new("author".into(), "author@example.com".into()).unwrap();

    // Minimal commit
    let c = minimal_commit();
    let mut enc = Vec::new();
    BinaryEncoder.encode_commit(&c, &mut enc).unwrap();
    let dec = BinaryDecoder.decode_commit(Cursor::new(&enc)).unwrap();
    assert_eq!(dec.tree(), c.tree());
    assert!(dec.parents().is_empty());
    assert_eq!(dec.author().name(), "author");
    assert_eq!(dec.message(), "message");

    // With parents
    let parents = vec![hash_from_byte(1), hash_from_byte(2), hash_from_byte(3)];
    let c = Commit::new(
        dummy_hash(),
        parents,
        user.clone(),
        user.clone(),
        "merge".into(),
    )
    .unwrap();
    let mut enc = Vec::new();
    BinaryEncoder.encode_commit(&c, &mut enc).unwrap();
    let dec = BinaryDecoder.decode_commit(Cursor::new(&enc)).unwrap();
    assert_eq!(dec.parents().len(), 3);

    // With many parents (u16 range — test 256 which exceeds old u8 limit)
    let many_parents: Vec<Hash> = (0u8..=255).map(hash_from_byte).collect();
    let c = Commit::new(
        dummy_hash(),
        many_parents.clone(),
        user.clone(),
        user.clone(),
        "octopus".into(),
    )
    .unwrap();
    let mut enc = Vec::new();
    BinaryEncoder.encode_commit(&c, &mut enc).unwrap();
    let dec = BinaryDecoder.decode_commit(Cursor::new(&enc)).unwrap();
    assert_eq!(dec.parents().len(), 256);
    assert_eq!(dec.parents(), many_parents);

    // Duplicate parent rejected
    let dup = vec![dummy_hash(), dummy_hash()];
    assert!(Commit::new(dummy_hash(), dup, user.clone(), user.clone(), "dup".into()).is_err());

    // Exceeds MAX_PARENT_COUNT rejected
    let too_many = vec![dummy_hash(); usize::try_from(MAX_PARENT_COUNT).unwrap() + 1];
    assert!(
        Commit::new(
            dummy_hash(),
            too_many,
            user.clone(),
            user.clone(),
            "toomany".into()
        )
        .is_err()
    );

    // With meta
    let meta = CommitMeta::new(1, 2, Some("UTF-8".into())).unwrap();
    let c = Commit::with_meta(
        dummy_hash(),
        vec![],
        user.clone(),
        user.clone(),
        "msg".into(),
        meta,
    )
    .unwrap();
    let mut enc = Vec::new();
    BinaryEncoder.encode_commit(&c, &mut enc).unwrap();
    let dec = BinaryDecoder.decode_commit(Cursor::new(&enc)).unwrap();
    assert_eq!(dec.meta().encoding(), Some("UTF-8"));

    // Invalid timezone offset
    assert!(CommitMeta::new(1, 1441, None).is_err());

    // Message too long
    let msg_len = usize::try_from(MAX_MESSAGE_LENGTH).unwrap() + 1;
    let msg = "A".repeat(msg_len);
    assert!(Commit::new(dummy_hash(), vec![], user.clone(), user, msg).is_err());
}

/// Tests tag encoding/decoding and limit enforcement.
///
/// Verifies:
/// - Lightweight tag round-trips.
/// - Annotated tag (with tagger and message) round-trips.
/// - Metadata encoding survives round-trip.
/// - Tag name longer than 255 bytes is rejected.
/// - Message exceeding `MAX_MESSAGE_LENGTH` is rejected.
#[test]
fn test_tag_roundtrip_and_limits() {
    let tagger = UserID::new("tagger".into(), "tag@example.com".into()).unwrap();

    // Lightweight tag
    let t = lightweight_tag("v0.1");
    let mut enc = Vec::new();
    BinaryEncoder.encode_tag(&t, &mut enc).unwrap();
    let dec = BinaryDecoder.decode_tag(Cursor::new(&enc)).unwrap();
    assert_eq!(dec.name(), "v0.1");
    assert!(dec.tagger().is_none());

    // Annotated tag
    let t = Tag::new(
        "v1.0".into(),
        dummy_hash(),
        Some(tagger.clone()),
        "Release".into(),
    )
    .unwrap();
    let mut enc = Vec::new();
    BinaryEncoder.encode_tag(&t, &mut enc).unwrap();
    let dec = BinaryDecoder.decode_tag(Cursor::new(&enc)).unwrap();
    assert_eq!(dec.tagger().unwrap().name(), "tagger");
    assert_eq!(dec.message(), "Release");

    // Tag with meta
    let meta = CommitMeta::new(3, 4, Some("ISO-8859-1".into())).unwrap();
    let t = Tag::with_meta(
        "v2.0".into(),
        dummy_hash(),
        Some(tagger),
        "msg".into(),
        meta,
    )
    .unwrap();
    let mut enc = Vec::new();
    BinaryEncoder.encode_tag(&t, &mut enc).unwrap();
    let dec = BinaryDecoder.decode_tag(Cursor::new(&enc)).unwrap();
    assert_eq!(dec.meta().encoding(), Some("ISO-8859-1"));

    // Tag name too long
    let long_name = "a".repeat(256);
    assert!(Tag::new(long_name, dummy_hash(), None, String::new()).is_err());

    // Message too long
    let msg_len = usize::try_from(MAX_MESSAGE_LENGTH).unwrap() + 1;
    let msg = "A".repeat(msg_len);
    assert!(Tag::new("v".into(), dummy_hash(), None, msg).is_err());
}

/// Tests that a corrupted version byte is rejected.
///
/// The version byte is the first byte of every encoded object. Changing it
/// to an unsupported value must cause decoding to fail with
/// [`VctrlError::CorruptedData`].
#[test]
fn test_wrong_version_rejected() {
    // Version 2 is no longer supported
    let b = Blob::new(vec![]).unwrap();
    let mut enc = Vec::new();
    BinaryEncoder.encode_blob(&b, &mut enc).unwrap();
    enc[0] = 0x02; // Corrupt version byte
    assert!(BinaryDecoder.decode_blob(Cursor::new(&enc)).is_err());
}
