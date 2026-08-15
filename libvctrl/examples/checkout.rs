//! Demonstrates a minimal checkout workflow using `libvctrl`.
//! Reads HEAD, loads commit, tree, and prints blob previews.

use std::io::Read;

use libvctrl::{
    BinaryDecoder, BinaryEncoder, Commit, Decoder, Encoder, EntryKind, Hasher, MemoryRefStore,
    MemoryStore, ObjectStore, RefStore, Sha512Hasher, Tree, TreeEntry, UserID, VctrlError,
};

fn main() -> Result<(), VctrlError> {
    // ---------------------------------------------------------------------
    // 0. Bootstrap a minimal repository in memory
    // ---------------------------------------------------------------------
    let mut obj_store = MemoryStore::new();
    let mut ref_store = MemoryRefStore::new();
    let encoder = BinaryEncoder;
    let decoder = BinaryDecoder;
    let hasher = Sha512Hasher;

    let user = UserID::new("Alice".into(), "alice@example.com".into())?;

    // --- Blob: README.md ---
    let readme_content = b"# My Project\n\nHello, world!";
    let readme_blob = libvctrl::Blob::new(readme_content.to_vec())?;
    let mut encoded_readme = Vec::new();
    encoder.encode_blob(&readme_blob, &mut encoded_readme)?;
    let readme_hash = hasher.hash(&encoded_readme[..])?;
    obj_store.put(&readme_hash, &encoded_readme)?;

    // --- Blob: src/main.rs ---
    let main_rs_content = b"fn main() { println!(\"Hello from libvctrl!\"); }";
    let main_blob = libvctrl::Blob::new(main_rs_content.to_vec())?;
    let mut encoded_main = Vec::new();
    encoder.encode_blob(&main_blob, &mut encoded_main)?;
    let main_hash = hasher.hash(&encoded_main[..])?;
    obj_store.put(&main_hash, &encoded_main)?;

    // --- Tree: src/ (subdirectory) ---
    let src_entry = TreeEntry::new("main.rs".into(), EntryKind::Blob, main_hash)?;
    let src_tree = Tree::new(vec![src_entry])?;
    let mut encoded_src_tree = Vec::new();
    encoder.encode_tree(&src_tree, &mut encoded_src_tree)?;
    let src_tree_hash = hasher.hash(&encoded_src_tree[..])?;
    obj_store.put(&src_tree_hash, &encoded_src_tree)?;

    // --- Root tree: README.md + src/ ---
    let readme_entry = TreeEntry::new("README.md".into(), EntryKind::Blob, readme_hash)?;
    let src_dir_entry = TreeEntry::new("src".into(), EntryKind::Tree, src_tree_hash)?;
    let root_tree = Tree::new(vec![readme_entry, src_dir_entry])?;
    let mut encoded_root_tree = Vec::new();
    encoder.encode_tree(&root_tree, &mut encoded_root_tree)?;
    let root_tree_hash = hasher.hash(&encoded_root_tree[..])?;
    obj_store.put(&root_tree_hash, &encoded_root_tree)?;

    // --- Commit ---
    let commit = Commit::new(
        root_tree_hash,
        vec![],
        user.clone(),
        user,
        "Initial commit".into(),
    )?;
    let mut encoded_commit = Vec::new();
    encoder.encode_commit(&commit, &mut encoded_commit)?;
    let commit_hash = hasher.hash(&encoded_commit[..])?;
    obj_store.put(&commit_hash, &encoded_commit)?;

    // Set HEAD
    ref_store.set_ref("HEAD", &commit_hash)?;

    // ---------------------------------------------------------------------
    // 1. Checkout: resolve HEAD → commit → tree → list entries
    // ---------------------------------------------------------------------
    println!("=== Checking out HEAD ===");
    let head_hash = ref_store.get_ref("HEAD")?;
    println!("HEAD commit hash: {head_hash}");

    // Load and decode the commit
    let mut encoded_commit = vec![];
    obj_store
        .get(&head_hash)?
        .read_to_end(&mut encoded_commit)
        .map_err(|e| VctrlError::IoError(std::sync::Arc::new(e)))?;
    let commit = decoder.decode_commit(&encoded_commit[..])?;
    println!("Commit message: {}", commit.message());

    // Load and decode the root tree
    let mut encoded_tree = vec![];
    obj_store
        .get(commit.tree())?
        .read_to_end(&mut encoded_tree)
        .map_err(|e| VctrlError::IoError(std::sync::Arc::new(e)))?;
    let tree = decoder.decode_tree(&encoded_tree[..])?;
    println!("Root tree contains {} entries:", tree.entries().len());

    for entry in tree.entries() {
        print!("  {:?} {}", entry.kind(), entry.name());
        if entry.kind() == EntryKind::Blob {
            let mut encoded_blob = vec![];
            obj_store
                .get(entry.hash())?
                .read_to_end(&mut encoded_blob)
                .map_err(|e| VctrlError::IoError(std::sync::Arc::new(e)))?;
            let blob = decoder.decode_blob(&encoded_blob[..])?;
            let preview = String::from_utf8_lossy(&blob.data()[..blob.data().len().min(60)]);
            println!(" → {}", preview.lines().next().unwrap_or(""));
        } else {
            println!();
        }
    }

    Ok(())
}
