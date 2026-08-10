//! Builds a complete repository with multiple commits, a branch, and a tag.

use libvctrl::{
    BinaryEncoder, Commit, Encoder, EntryKind, Hasher, MemoryRefStore, MemoryStore, ObjectStore,
    RefStore, Sha512Hasher, Tag, Tree, TreeEntry, UserID, VctrlError,
};

fn main() -> Result<(), VctrlError> {
    let mut obj_store = MemoryStore::new();
    let mut ref_store = MemoryRefStore::new();
    let encoder = BinaryEncoder;
    let hasher = Sha512Hasher;

    let alice = UserID::new("Alice".into(), "alice@example.com".into())?;
    let bob = UserID::new("Bob".into(), "bob@example.com".into())?;

    // ---- Commit 1: Initial commit (README.md only) ----
    let readme_blob = libvctrl::Blob::new(b"# My Project\n\nHello, world!".to_vec());
    let encoded_readme = encoder.encode_blob(&readme_blob)?;
    let readme_hash = hasher.hash(&encoded_readme);
    obj_store.put(&readme_hash, &encoded_readme)?;

    let root1 = Tree::new(vec![TreeEntry::new(
        "README.md".into(),
        EntryKind::Blob,
        readme_hash,
    )?])?;
    let encoded_root1 = encoder.encode_tree(&root1)?;
    let root1_hash = hasher.hash(&encoded_root1);
    obj_store.put(&root1_hash, &encoded_root1)?;

    let c1 = {
        let commit = Commit::new(
            root1_hash,
            vec![],
            alice.clone(),
            alice.clone(),
            "Initial commit".into(),
        );
        let enc = encoder.encode_commit(&commit)?;
        let hash = hasher.hash(&enc);
        obj_store.put(&hash, &enc)?;
        hash
    };

    ref_store.set_ref("refs/heads/main", &c1)?;
    ref_store.set_ref("HEAD", &c1)?;

    // ---- Commit 2: Add src/main.rs ----
    let main_blob =
        libvctrl::Blob::new(b"fn main() { println!(\"Hello from libvctrl!\"); }".to_vec());
    let encoded_main = encoder.encode_blob(&main_blob)?;
    let main_hash = hasher.hash(&encoded_main);
    obj_store.put(&main_hash, &encoded_main)?;

    let root2 = Tree::new(vec![
        TreeEntry::new("README.md".into(), EntryKind::Blob, readme_hash)?,
        TreeEntry::new("main.rs".into(), EntryKind::Blob, main_hash)?,
    ])?;
    let encoded_root2 = encoder.encode_tree(&root2)?;
    let root2_hash = hasher.hash(&encoded_root2);
    obj_store.put(&root2_hash, &encoded_root2)?;

    let c2 = {
        let commit = Commit::new(root2_hash, vec![c1], bob.clone(), bob, "Add main.rs".into());
        let enc = encoder.encode_commit(&commit)?;
        let hash = hasher.hash(&enc);
        obj_store.put(&hash, &enc)?;
        hash
    };

    ref_store.set_ref("refs/heads/main", &c2)?;
    ref_store.set_ref("HEAD", &c2)?;

    // ---- Tag v1.0 on first commit ----
    let _tag = Tag::new("v1.0".into(), c1, Some(alice), "First release".into())?;
    ref_store.set_ref("refs/tags/v1.0", &c1)?;

    println!("Database built successfully!");
    println!("  Commit 1 (initial): {c1}");
    println!("  Commit 2 (add main): {c2}");
    println!("  HEAD -> refs/heads/main -> commit 2");
    println!("  Tag v1.0 -> commit 1");
    Ok(())
}
