//! Integration tests for the cat-file plumbing command.

use libvctrl::{BinaryDecoder, BinaryEncoder};
use libvctrl::{
    Blob, Commit, Encoder, EntryKind, Hash, Hasher, ObjectStore, Tag, Tree, TreeEntry, UserID,
    VctrlError,
};
use libvctrl::{MemoryStore, Sha512Hasher};
use libvctrl_core as _;
use libvctrl_plumbing::cat_file::{
    BatchOptions, CatFileMode, ObjectType, cat_file, cat_file_batch,
};
use std::io::Cursor;

// Helper: build a minimal repository with one object of each type
struct TestRepo {
    store: MemoryStore,
    decoder: BinaryDecoder,
    blob_hash: Hash,
    tree_hash: Hash,
    commit_hash: Hash,
    tag_hash: Hash,
}

impl TestRepo {
    fn new() -> Result<Self, VctrlError> {
        let mut store = MemoryStore::new();
        let encoder = BinaryEncoder;
        let hasher = Sha512Hasher;

        let user = UserID::new("Alice".into(), "alice@example.com".into())?;

        // Blob
        let blob = Blob::new(b"hello, world\n".to_vec())?;
        let mut enc_blob = Vec::new();
        encoder.encode_blob(&blob, &mut enc_blob)?;
        let blob_hash = hasher.hash(enc_blob.as_slice())?;
        store.put(&blob_hash, &enc_blob)?;

        // Tree with one entry pointing to the blob
        let entry = TreeEntry::new("file.txt".into(), EntryKind::Blob, blob_hash)?;
        let tree = Tree::new(vec![entry])?;
        let mut enc_tree = Vec::new();
        encoder.encode_tree(&tree, &mut enc_tree)?;
        let tree_hash = hasher.hash(enc_tree.as_slice())?;
        store.put(&tree_hash, &enc_tree)?;

        // Commit
        let commit = Commit::new(
            tree_hash,
            vec![],
            user.clone(),
            user.clone(),
            "Initial commit\n".into(),
        )?;
        let mut enc_commit = Vec::new();
        encoder.encode_commit(&commit, &mut enc_commit)?;
        let commit_hash = hasher.hash(enc_commit.as_slice())?;
        store.put(&commit_hash, &enc_commit)?;

        // Tag pointing to the commit
        let tag = Tag::new("v1.0".into(), commit_hash, Some(user), "Release\n".into())?;
        let mut enc_tag = Vec::new();
        encoder.encode_tag(&tag, &mut enc_tag)?;
        let tag_hash = hasher.hash(enc_tag.as_slice())?;
        store.put(&tag_hash, &enc_tag)?;

        Ok(Self {
            store,
            decoder: BinaryDecoder,
            blob_hash,
            tree_hash,
            commit_hash,
            tag_hash,
        })
    }
}

fn utf8_string(bytes: Vec<u8>) -> Result<String, VctrlError> {
    String::from_utf8(bytes)
        .map_err(|e| VctrlError::Other(format!("invalid UTF-8 in test output: {e}")))
}

#[test]
fn exists_valid_object() -> Result<(), VctrlError> {
    let repo = TestRepo::new()?;
    let mut out = Vec::new();
    cat_file(
        &repo.store,
        &repo.decoder,
        &repo.blob_hash.to_string(),
        CatFileMode::Exists,
        &mut out,
    )?;
    assert!(out.is_empty());
    Ok(())
}

#[test]
fn exists_invalid_hash() -> Result<(), VctrlError> {
    let repo = TestRepo::new()?;
    let mut out = Vec::new();
    let result = cat_file(
        &repo.store,
        &repo.decoder,
        "not a hash",
        CatFileMode::Exists,
        &mut out,
    );
    assert!(result.is_err());
    Ok(())
}

#[test]
fn object_type_blob() -> Result<(), VctrlError> {
    let repo = TestRepo::new()?;
    let mut out = Vec::new();
    cat_file(
        &repo.store,
        &repo.decoder,
        &repo.blob_hash.to_string(),
        CatFileMode::ObjectType,
        &mut out,
    )?;
    assert_eq!(utf8_string(out)?.trim(), "blob");
    Ok(())
}

#[test]
fn object_type_tree() -> Result<(), VctrlError> {
    let repo = TestRepo::new()?;
    let mut out = Vec::new();
    cat_file(
        &repo.store,
        &repo.decoder,
        &repo.tree_hash.to_string(),
        CatFileMode::ObjectType,
        &mut out,
    )?;
    assert_eq!(utf8_string(out)?.trim(), "tree");
    Ok(())
}

#[test]
fn object_type_commit() -> Result<(), VctrlError> {
    let repo = TestRepo::new()?;
    let mut out = Vec::new();
    cat_file(
        &repo.store,
        &repo.decoder,
        &repo.commit_hash.to_string(),
        CatFileMode::ObjectType,
        &mut out,
    )?;
    assert_eq!(utf8_string(out)?.trim(), "commit");
    Ok(())
}

#[test]
fn object_type_tag() -> Result<(), VctrlError> {
    let repo = TestRepo::new()?;
    let mut out = Vec::new();
    cat_file(
        &repo.store,
        &repo.decoder,
        &repo.tag_hash.to_string(),
        CatFileMode::ObjectType,
        &mut out,
    )?;
    assert_eq!(utf8_string(out)?.trim(), "tag");
    Ok(())
}

#[test]
fn object_size() -> Result<(), VctrlError> {
    let repo = TestRepo::new()?;
    let mut out = Vec::new();
    cat_file(
        &repo.store,
        &repo.decoder,
        &repo.blob_hash.to_string(),
        CatFileMode::ObjectSize,
        &mut out,
    )?;
    let size: usize = utf8_string(out)?
        .trim()
        .parse::<usize>()
        .map_err(|e: std::num::ParseIntError| VctrlError::Other(e.to_string()))?;
    assert!(size > 0);
    Ok(())
}

#[test]
fn pretty_print_blob() -> Result<(), VctrlError> {
    let repo = TestRepo::new()?;
    let mut out = Vec::new();
    cat_file(
        &repo.store,
        &repo.decoder,
        &repo.blob_hash.to_string(),
        CatFileMode::PrettyPrint,
        &mut out,
    )?;
    let text = utf8_string(out)?;
    assert!(text.contains("hello, world"));
    Ok(())
}

#[test]
fn pretty_print_tree() -> Result<(), VctrlError> {
    let repo = TestRepo::new()?;
    let mut out = Vec::new();
    cat_file(
        &repo.store,
        &repo.decoder,
        &repo.tree_hash.to_string(),
        CatFileMode::PrettyPrint,
        &mut out,
    )?;
    let text = utf8_string(out)?;
    assert!(text.contains("file.txt"));
    assert!(text.contains("100644"));
    Ok(())
}

#[test]
fn pretty_print_commit() -> Result<(), VctrlError> {
    let repo = TestRepo::new()?;
    let mut out = Vec::new();
    cat_file(
        &repo.store,
        &repo.decoder,
        &repo.commit_hash.to_string(),
        CatFileMode::PrettyPrint,
        &mut out,
    )?;
    let text = utf8_string(out)?;
    assert!(text.contains("tree "));
    assert!(text.contains("author Alice <alice@example.com>"));
    assert!(text.contains("Initial commit"));
    Ok(())
}

#[test]
fn pretty_print_tag() -> Result<(), VctrlError> {
    let repo = TestRepo::new()?;
    let mut out = Vec::new();
    cat_file(
        &repo.store,
        &repo.decoder,
        &repo.tag_hash.to_string(),
        CatFileMode::PrettyPrint,
        &mut out,
    )?;
    let text = utf8_string(out)?;
    assert!(text.contains("object "));
    assert!(text.contains("tag v1.0"));
    assert!(text.contains("Release"));
    Ok(())
}

#[test]
fn raw_with_correct_type() -> Result<(), VctrlError> {
    let repo = TestRepo::new()?;
    let mut out = Vec::new();
    cat_file(
        &repo.store,
        &repo.decoder,
        &repo.blob_hash.to_string(),
        CatFileMode::Raw(ObjectType::Blob),
        &mut out,
    )?;
    let encoder = BinaryEncoder;
    let blob = Blob::new(b"hello, world\n".to_vec())?;
    let mut expected = Vec::new();
    encoder.encode_blob(&blob, &mut expected)?;
    assert_eq!(out, expected);
    Ok(())
}

#[test]
fn raw_with_wrong_type_errors() -> Result<(), VctrlError> {
    let repo = TestRepo::new()?;
    let mut out = Vec::new();
    let result = cat_file(
        &repo.store,
        &repo.decoder,
        &repo.blob_hash.to_string(),
        CatFileMode::Raw(ObjectType::Tree),
        &mut out,
    );
    match result {
        Err(VctrlError::Other(msg)) => assert!(msg.contains("is a blob, not a tree")),
        other => {
            return Err(VctrlError::Other(format!(
                "expected VctrlError::Other with type mismatch, got {other:?}"
            )));
        }
    }
    Ok(())
}

#[test]
fn batch_check_single_object() -> Result<(), VctrlError> {
    let repo = TestRepo::new()?;
    let input = format!("{0}\n", repo.blob_hash);
    let mut output = Vec::new();
    let options = BatchOptions::default();
    cat_file_batch(
        &repo.store,
        &repo.decoder,
        &mut Cursor::new(input),
        &mut output,
        &options,
    )?;
    let out_str = utf8_string(output)?;
    let expected_prefix = format!("{0} blob", repo.blob_hash);
    assert!(out_str.starts_with(&expected_prefix));
    assert!(out_str.ends_with('\n'));
    Ok(())
}

#[test]
fn batch_with_contents() -> Result<(), VctrlError> {
    let repo = TestRepo::new()?;
    let input = format!("{0}\n", repo.blob_hash);
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
    )?;
    let out_str = utf8_string(output)?;
    assert!(out_str.contains("hello, world"));
    assert!(out_str.lines().count() >= 2);
    Ok(())
}

#[test]
fn batch_with_buffer() -> Result<(), VctrlError> {
    let repo = TestRepo::new()?;
    let input = format!("{0}\n{1}\n", repo.blob_hash, repo.tree_hash);
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
    )?;
    let out_str = utf8_string(output)?;
    assert!(out_str.contains(&repo.blob_hash.to_string()));
    assert!(out_str.contains(&repo.tree_hash.to_string()));
    Ok(())
}

#[test]
fn batch_nul_terminated() -> Result<(), VctrlError> {
    let repo = TestRepo::new()?;
    let input = format!("{0}\0{1}\0", repo.blob_hash, repo.tree_hash);
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
    )?;
    assert!(output.ends_with(&[0u8]));
    let out_str = String::from_utf8_lossy(&output);
    assert!(out_str.contains(&repo.blob_hash.to_string()));
    assert!(out_str.contains(&repo.tree_hash.to_string()));
    Ok(())
}

#[test]
fn batch_custom_format() -> Result<(), VctrlError> {
    let repo = TestRepo::new()?;
    let input = format!("{0}\n", repo.blob_hash);
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
    )?;
    let out_str = utf8_string(output)?.trim().to_string();
    let expected = format!("{0} blob", repo.blob_hash);
    assert_eq!(out_str, expected);
    Ok(())
}

#[test]
fn batch_missing_object() -> Result<(), VctrlError> {
    let repo = TestRepo::new()?;
    let fake_hash = "a".repeat(128);
    let input = format!("{fake_hash}\n");
    let mut output = Vec::new();
    let options = BatchOptions::default();
    cat_file_batch(
        &repo.store,
        &repo.decoder,
        &mut Cursor::new(input),
        &mut output,
        &options,
    )?;
    let out_str = utf8_string(output)?.trim().to_string();
    assert!(out_str.ends_with("missing"));
    Ok(())
}

#[test]
fn batch_empty_input() -> Result<(), VctrlError> {
    let repo = TestRepo::new()?;
    let input = "";
    let mut output = Vec::new();
    let options = BatchOptions::default();
    cat_file_batch(
        &repo.store,
        &repo.decoder,
        &mut Cursor::new(input),
        &mut output,
        &options,
    )?;
    assert!(output.is_empty());
    Ok(())
}
