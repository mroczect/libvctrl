//! Integration tests for the cat-file plumbing command.
//!
//! These tests exercise every mode (exists, type, size, pretty‑print, raw)
//! as well as the batch processing with various options (buffer, NUL
//! termination, custom format, missing objects).

use libvctrl::{BinaryDecoder, BinaryEncoder};
use libvctrl::{
    Blob, Commit, Encoder, EntryKind, Hash, Hasher, ObjectStore, Tag, Tree, TreeEntry, UserID,
    VctrlError,
};
use libvctrl::{MemoryStore, Sha512Hasher};
use libvctrl_plumbing::cat_file::{
    BatchOptions, CatFileMode, ObjectType, cat_file, cat_file_batch,
};
use std::io::Cursor;

// ------------------------------------------------------------------
// Helper: build a minimal repository with one object of each type
// ------------------------------------------------------------------
struct TestRepo {
    store: MemoryStore,
    decoder: BinaryDecoder,
    blob_hash: Hash,
    tree_hash: Hash,
    commit_hash: Hash,
    tag_hash: Hash,
}

impl TestRepo {
    fn new() -> Self {
        let mut store = MemoryStore::new();
        let encoder = BinaryEncoder;
        let hasher = Sha512Hasher;
        let user = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();

        // Blob
        let blob = Blob::new(b"hello, world\n".to_vec());
        let enc_blob = encoder.encode_blob(&blob).unwrap();
        let blob_hash = hasher.hash(&enc_blob).unwrap();
        store.put(&blob_hash, &enc_blob).unwrap();

        // Tree with one entry pointing to the blob
        let entry = TreeEntry::new("file.txt".into(), EntryKind::Blob, blob_hash).unwrap();
        let tree = Tree::new(vec![entry]).unwrap();
        let enc_tree = encoder.encode_tree(&tree).unwrap();
        let tree_hash = hasher.hash(&enc_tree).unwrap();
        store.put(&tree_hash, &enc_tree).unwrap();

        // Commit
        let commit = Commit::new(
            tree_hash,
            vec![],
            user.clone(),
            user.clone(),
            "Initial commit\n".into(),
        );
        let enc_commit = encoder.encode_commit(&commit).unwrap();
        let commit_hash = hasher.hash(&enc_commit).unwrap();
        store.put(&commit_hash, &enc_commit).unwrap();

        // Tag pointing to the commit
        let tag = Tag::new("v1.0".into(), commit_hash, Some(user), "Release\n".into()).unwrap();
        let enc_tag = encoder.encode_tag(&tag).unwrap();
        let tag_hash = hasher.hash(&enc_tag).unwrap();
        store.put(&tag_hash, &enc_tag).unwrap();

        Self {
            store,
            decoder: BinaryDecoder,
            blob_hash,
            tree_hash,
            commit_hash,
            tag_hash,
        }
    }
}

// ---------------------------------------------------------------
// Non‑batch mode tests
// ---------------------------------------------------------------

#[test]
fn exists_valid_object() {
    let repo = TestRepo::new();
    let mut out = Vec::new();
    cat_file(
        &repo.store,
        &repo.decoder,
        &repo.blob_hash.to_string(),
        CatFileMode::Exists,
        &mut out,
    )
    .unwrap();
    assert!(out.is_empty());
}

#[test]
fn exists_invalid_hash() {
    let repo = TestRepo::new();
    let mut out = Vec::new();
    let result = cat_file(
        &repo.store,
        &repo.decoder,
        "not a hash",
        CatFileMode::Exists,
        &mut out,
    );
    assert!(result.is_err());
}

#[test]
fn object_type_blob() {
    let repo = TestRepo::new();
    let mut out = Vec::new();
    cat_file(
        &repo.store,
        &repo.decoder,
        &repo.blob_hash.to_string(),
        CatFileMode::ObjectType,
        &mut out,
    )
    .unwrap();
    assert_eq!(String::from_utf8(out).unwrap().trim(), "blob");
}

#[test]
fn object_type_tree() {
    let repo = TestRepo::new();
    let mut out = Vec::new();
    cat_file(
        &repo.store,
        &repo.decoder,
        &repo.tree_hash.to_string(),
        CatFileMode::ObjectType,
        &mut out,
    )
    .unwrap();
    assert_eq!(String::from_utf8(out).unwrap().trim(), "tree");
}

#[test]
fn object_type_commit() {
    let repo = TestRepo::new();
    let mut out = Vec::new();
    cat_file(
        &repo.store,
        &repo.decoder,
        &repo.commit_hash.to_string(),
        CatFileMode::ObjectType,
        &mut out,
    )
    .unwrap();
    assert_eq!(String::from_utf8(out).unwrap().trim(), "commit");
}

#[test]
fn object_type_tag() {
    let repo = TestRepo::new();
    let mut out = Vec::new();
    cat_file(
        &repo.store,
        &repo.decoder,
        &repo.tag_hash.to_string(),
        CatFileMode::ObjectType,
        &mut out,
    )
    .unwrap();
    assert_eq!(String::from_utf8(out).unwrap().trim(), "tag");
}

#[test]
fn object_size() {
    let repo = TestRepo::new();
    let mut out = Vec::new();
    cat_file(
        &repo.store,
        &repo.decoder,
        &repo.blob_hash.to_string(),
        CatFileMode::ObjectSize,
        &mut out,
    )
    .unwrap();
    let size: usize = String::from_utf8(out).unwrap().trim().parse().unwrap();
    assert!(size > 0);
}

#[test]
fn pretty_print_blob() {
    let repo = TestRepo::new();
    let mut out = Vec::new();
    cat_file(
        &repo.store,
        &repo.decoder,
        &repo.blob_hash.to_string(),
        CatFileMode::PrettyPrint,
        &mut out,
    )
    .unwrap();
    let text = String::from_utf8(out).unwrap();
    assert!(text.contains("hello, world"));
}

#[test]
fn pretty_print_tree() {
    let repo = TestRepo::new();
    let mut out = Vec::new();
    cat_file(
        &repo.store,
        &repo.decoder,
        &repo.tree_hash.to_string(),
        CatFileMode::PrettyPrint,
        &mut out,
    )
    .unwrap();
    let text = String::from_utf8(out).unwrap();
    assert!(text.contains("file.txt"));
    assert!(text.contains("100644"));
}

#[test]
fn pretty_print_commit() {
    let repo = TestRepo::new();
    let mut out = Vec::new();
    cat_file(
        &repo.store,
        &repo.decoder,
        &repo.commit_hash.to_string(),
        CatFileMode::PrettyPrint,
        &mut out,
    )
    .unwrap();
    let text = String::from_utf8(out).unwrap();
    assert!(text.contains("tree "));
    assert!(text.contains("author Alice <alice@example.com>"));
    assert!(text.contains("Initial commit"));
}

#[test]
fn pretty_print_tag() {
    let repo = TestRepo::new();
    let mut out = Vec::new();
    cat_file(
        &repo.store,
        &repo.decoder,
        &repo.tag_hash.to_string(),
        CatFileMode::PrettyPrint,
        &mut out,
    )
    .unwrap();
    let text = String::from_utf8(out).unwrap();
    assert!(text.contains("object "));
    assert!(text.contains("tag v1.0"));
    assert!(text.contains("Release"));
}

#[test]
fn raw_with_correct_type() {
    let repo = TestRepo::new();
    let mut out = Vec::new();
    cat_file(
        &repo.store,
        &repo.decoder,
        &repo.blob_hash.to_string(),
        CatFileMode::Raw(ObjectType::Blob),
        &mut out,
    )
    .unwrap();
    let encoder = BinaryEncoder;
    let blob = Blob::new(b"hello, world\n".to_vec());
    let expected = encoder.encode_blob(&blob).unwrap();
    assert_eq!(out, expected);
}

#[test]
fn raw_with_wrong_type_errors() {
    let repo = TestRepo::new();
    let mut out = Vec::new();
    let result = cat_file(
        &repo.store,
        &repo.decoder,
        &repo.blob_hash.to_string(),
        CatFileMode::Raw(ObjectType::Tree),
        &mut out,
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    if let VctrlError::Other(msg) = err {
        assert!(msg.contains("is a blob, not a tree"));
    } else {
        panic!("Expected VctrlError::Other");
    }
}

// ---------------------------------------------------------------
// Batch mode tests
// ---------------------------------------------------------------

#[test]
fn batch_check_single_object() {
    let repo = TestRepo::new();
    let input = format!("{}\n", repo.blob_hash);
    let mut output = Vec::new();
    let options = BatchOptions::default();
    cat_file_batch(
        &repo.store,
        &repo.decoder,
        &mut Cursor::new(input),
        &mut output,
        &options,
    )
    .unwrap();
    let out_str = String::from_utf8(output).unwrap();
    let expected_prefix = format!("{} blob", repo.blob_hash);
    assert!(out_str.starts_with(&expected_prefix));
    assert!(out_str.ends_with('\n'));
}

#[test]
fn batch_with_contents() {
    let repo = TestRepo::new();
    let input = format!("{}\n", repo.blob_hash);
    let mut output = Vec::new();
    let options = BatchOptions {
        print_contents: true,
        ..Default::default()
    };
    cat_file_batch(
        &repo.store,
        &repo.decoder,
        &mut Cursor::new(input),
        &mut output,
        &options,
    )
    .unwrap();
    let out_str = String::from_utf8(output).unwrap();
    assert!(out_str.contains("hello, world"));
    assert!(out_str.lines().count() >= 2);
}

#[test]
fn batch_with_buffer() {
    let repo = TestRepo::new();
    let input = format!("{}\n{}\n", repo.blob_hash, repo.tree_hash);
    let mut output = Vec::new();
    let options = BatchOptions {
        buffer: true,
        ..Default::default()
    };
    cat_file_batch(
        &repo.store,
        &repo.decoder,
        &mut Cursor::new(input),
        &mut output,
        &options,
    )
    .unwrap();
    let out_str = String::from_utf8(output).unwrap();
    assert!(out_str.contains(&repo.blob_hash.to_string()));
    assert!(out_str.contains(&repo.tree_hash.to_string()));
}

#[test]
fn batch_nul_terminated() {
    let repo = TestRepo::new();
    let input = format!("{}\0{}\0", repo.blob_hash, repo.tree_hash);
    let mut output = Vec::new();
    let options = BatchOptions {
        nul_terminated: true,
        ..Default::default()
    };
    cat_file_batch(
        &repo.store,
        &repo.decoder,
        &mut Cursor::new(input),
        &mut output,
        &options,
    )
    .unwrap();
    assert!(output.ends_with(&[0u8]));
    let out_str = String::from_utf8_lossy(&output);
    assert!(out_str.contains(&repo.blob_hash.to_string()));
    assert!(out_str.contains(&repo.tree_hash.to_string()));
}

#[test]
fn batch_custom_format() {
    let repo = TestRepo::new();
    let input = format!("{}\n", repo.blob_hash);
    let mut output = Vec::new();
    let options = BatchOptions {
        format: Some("%(objectname) %(objecttype)".into()),
        ..Default::default()
    };
    cat_file_batch(
        &repo.store,
        &repo.decoder,
        &mut Cursor::new(input),
        &mut output,
        &options,
    )
    .unwrap();
    let out_str = String::from_utf8(output).unwrap().trim().to_string();
    let expected = format!("{} blob", repo.blob_hash);
    assert_eq!(out_str, expected);
}

#[test]
fn batch_missing_object() {
    let repo = TestRepo::new();
    let fake_hash = "a".repeat(128);
    let input = format!("{}\n", fake_hash);
    let mut output = Vec::new();
    let options = BatchOptions::default();
    cat_file_batch(
        &repo.store,
        &repo.decoder,
        &mut Cursor::new(input),
        &mut output,
        &options,
    )
    .unwrap();
    let out_str = String::from_utf8(output).unwrap().trim().to_string();
    assert!(out_str.ends_with("missing"));
}

#[test]
fn batch_empty_input() {
    let repo = TestRepo::new();
    let input = "";
    let mut output = Vec::new();
    let options = BatchOptions::default();
    cat_file_batch(
        &repo.store,
        &repo.decoder,
        &mut Cursor::new(input),
        &mut output,
        &options,
    )
    .unwrap();
    assert!(output.is_empty());
}
