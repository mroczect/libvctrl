//! Reads a repository and displays commit log, tree, and blob previews.

use libvctrl::{
    BinaryDecoder, BinaryEncoder, Commit, Decoder, Encoder, EntryKind, Hasher, MemoryRefStore,
    MemoryStore, ObjectStore, RefStore, Sha512Hasher, Tree, TreeEntry, UserID, VctrlError,
};
use std::io::Read;

/// Builds an example database and returns (`object_store`, `ref_store`)
fn build_database() -> Result<(MemoryStore, MemoryRefStore), VctrlError> {
    let mut obj_store = MemoryStore::new();
    let mut ref_store = MemoryRefStore::new();
    let encoder = BinaryEncoder;
    let hasher = Sha512Hasher;
    let alice = UserID::new("Alice".into(), "alice@example.com".into())?;
    let bob = UserID::new("Bob".into(), "bob@example.com".into())?;

    let readme_blob = libvctrl::Blob::new(b"# My Project\n\nHello, world!".to_vec());
    let readme_enc = encoder.encode_blob(&readme_blob)?;
    let readme_hash = hasher.hash(&readme_enc);
    obj_store.put(&readme_hash, &readme_enc)?;

    let root1 = Tree::new(vec![TreeEntry::new(
        "README.md".into(),
        EntryKind::Blob,
        readme_hash,
    )?])?;
    let root1_enc = encoder.encode_tree(&root1)?;
    let root1_hash = hasher.hash(&root1_enc);
    obj_store.put(&root1_hash, &root1_enc)?;

    let c1 = {
        let commit = Commit::new(
            root1_hash,
            vec![],
            alice.clone(),
            alice,
            "Initial commit".into(),
        );
        let enc = encoder.encode_commit(&commit)?;
        let h = hasher.hash(&enc);
        obj_store.put(&h, &enc)?;
        h
    };

    let main_blob =
        libvctrl::Blob::new(b"fn main() { println!(\"Hello from libvctrl!\"); }".to_vec());
    let main_enc = encoder.encode_blob(&main_blob)?;
    let main_hash = hasher.hash(&main_enc);
    obj_store.put(&main_hash, &main_enc)?;

    let root2 = Tree::new(vec![
        TreeEntry::new("README.md".into(), EntryKind::Blob, readme_hash)?,
        TreeEntry::new("main.rs".into(), EntryKind::Blob, main_hash)?,
    ])?;
    let root2_enc = encoder.encode_tree(&root2)?;
    let root2_hash = hasher.hash(&root2_enc);
    obj_store.put(&root2_hash, &root2_enc)?;

    let c2 = {
        let commit = Commit::new(root2_hash, vec![c1], bob.clone(), bob, "Add main.rs".into());
        let enc = encoder.encode_commit(&commit)?;
        let h = hasher.hash(&enc);
        obj_store.put(&h, &enc)?;
        h
    };

    ref_store.set_ref("HEAD", &c2)?;
    ref_store.set_ref("refs/heads/main", &c2)?;
    Ok((obj_store, ref_store))
}

/// Prints the commit log starting from HEAD
fn print_commit_log(obj_store: &MemoryStore, ref_store: &MemoryRefStore) -> Result<(), VctrlError> {
    let decoder = BinaryDecoder;
    println!("=== Commit Log (HEAD) ===\n");
    let mut current_hash = ref_store.get_ref("HEAD")?;
    loop {
        let mut encoded = vec![];
        obj_store
            .get(&current_hash)?
            .read_to_end(&mut encoded)
            .map_err(VctrlError::IoError)?;
        let commit = decoder.decode_commit(&encoded)?;
        println!("commit {current_hash}");
        println!("  Author:  {}", commit.author().name());
        println!("  Message: {}\n", commit.message());
        if commit.parents().is_empty() {
            break;
        }
        current_hash = commit.parents()[0];
    }
    Ok(())
}

/// Prints the tree and blob previews at HEAD
fn print_tree(obj_store: &MemoryStore, ref_store: &MemoryRefStore) -> Result<(), VctrlError> {
    let decoder = BinaryDecoder;
    println!("=== Tree at HEAD ===\n");
    let mut encoded_commit = vec![];
    obj_store
        .get(&ref_store.get_ref("HEAD")?)?
        .read_to_end(&mut encoded_commit)
        .map_err(VctrlError::IoError)?;
    let head_commit = decoder.decode_commit(&encoded_commit)?;
    let mut encoded_tree = vec![];
    obj_store
        .get(head_commit.tree())?
        .read_to_end(&mut encoded_tree)
        .map_err(VctrlError::IoError)?;
    let tree = decoder.decode_tree(&encoded_tree)?;

    for entry in tree.entries() {
        print!("  {:?} {}", entry.kind(), entry.name());
        if entry.kind() == EntryKind::Blob {
            let mut encoded_blob = vec![];
            obj_store
                .get(entry.hash())?
                .read_to_end(&mut encoded_blob)
                .map_err(VctrlError::IoError)?;
            let blob = decoder.decode_blob(&encoded_blob)?;
            let preview = String::from_utf8_lossy(&blob.data()[..blob.data().len().min(40)]);
            println!(" → {}", preview.lines().next().unwrap_or(""));
        } else {
            println!();
        }
    }
    Ok(())
}

fn main() -> Result<(), VctrlError> {
    let (obj_store, ref_store) = build_database()?;
    print_commit_log(&obj_store, &ref_store)?;
    print_tree(&obj_store, &ref_store)?;
    Ok(())
}
