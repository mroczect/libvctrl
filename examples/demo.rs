#[allow(clippy::wildcard_imports)]
use libvctrl::*;

fn hash_of_commit(commit: &Commit) -> Hash {
    let encoder = BinaryEncoder;
    let hasher = Sha512Hasher;
    let mut buf = Vec::new();
    encoder.encode_commit(commit, &mut buf).unwrap();
    hasher.hash_commit_encoded(&buf)
}

struct DummyVerifier;
impl Verifier for DummyVerifier {
    fn verify(&self, _data: &[u8], _sig: &[u8]) -> Result<bool, VctrlError> {
        Ok(true)
    }
}

struct OursResolver;
impl ConflictResolver for OursResolver {
    fn resolve(&self, _base: &[u8], ours: &[u8], _theirs: &[u8]) -> Option<Vec<u8>> {
        Some(ours.to_vec())
    }
}

fn main() -> Result<(), VctrlError> {
    let mut store = MemoryStore::new();
    let mut refs = MemoryRefStore::new();

    let alice = UserID::new("Alice".into(), "alice@example.com".into())?;
    let bob = UserID::new("Bob".into(), "bob@example.com".into())?;

    println!("--- 1. Init ---");
    let init_cmd = Init {
        author: alice.clone(),
        encoder: Box::new(BinaryEncoder),
        hasher: Box::new(Sha512Hasher),
    };
    let init_commit_hash = init_cmd.execute(&mut store, &mut refs)?;
    println!(
        "Repository initialized. Initial commit: {}",
        init_commit_hash
    );

    println!("--- 2. Create blob, tree, and second commit ---");
    let readme_bytes = b"Hello, libvctrl!";
    let readme_blob = Blob::new(readme_bytes.to_vec());
    let readme_hash = Sha512Hasher.hash_blob(readme_bytes);
    store.put(&readme_hash, &Object::Blob(readme_blob))?;
    println!("Blob for README.md created (hash: {})", readme_hash);

    let readme_entry = TreeEntry::new("README.md".to_string(), EntryKind::Blob, readme_hash)
        .map_err(VctrlError::Tree)?;
    let tree1 = Tree::new(vec![readme_entry]).map_err(VctrlError::Tree)?;
    let mut tree1_buf = Vec::new();
    BinaryEncoder.encode_tree(&tree1, &mut tree1_buf)?;
    let tree1_hash = Sha512Hasher.hash_tree_encoded(&tree1_buf);
    store.put(&tree1_hash, &Object::Tree(tree1))?;
    println!("Tree created (hash: {})", tree1_hash);

    let commit1_cmd = CreateCommit {
        tree_hash: tree1_hash,
        parents: vec![],
        author: alice.clone(),
        committer: alice.clone(),
        message: "Second commit – add README.md".into(),
        encoder: Box::new(BinaryEncoder),
        hasher: Box::new(Sha512Hasher),
    };
    let commit1_hash = commit1_cmd.execute(&mut store, &mut refs)?;
    println!("Second commit created: {}", commit1_hash);

    println!("--- 3. Log ---");
    let log = Log.execute(&mut store, &mut refs)?;
    for (i, c) in log.iter().enumerate() {
        let h = hash_of_commit(c);
        println!("  [{}] {} {:?}", i, &h.to_hex()[..8], c.message);
    }

    println!("--- 4. Branching ---");
    let feature_branch = "refs/heads/feature";
    CreateBranch {
        name: feature_branch.to_string(),
        hash: commit1_hash,
    }
    .execute(&mut store, &mut refs)?;
    println!("Created branch '{}'", feature_branch);

    SetHead {
        target: feature_branch.to_string(),
    }
    .execute(&mut store, &mut refs)?;
    println!("HEAD now at '{}'", feature_branch);

    let feature_bytes = b"Feature work";
    let feature_blob = Blob::new(feature_bytes.to_vec());
    let feature_blob_hash = Sha512Hasher.hash_blob(feature_bytes);
    store.put(&feature_blob_hash, &Object::Blob(feature_blob))?;
    let feature_entry = TreeEntry::new(
        "feature.txt".to_string(),
        EntryKind::Blob,
        feature_blob_hash,
    )
    .map_err(VctrlError::Tree)?;
    let feature_tree = Tree::new(vec![feature_entry]).map_err(VctrlError::Tree)?;
    let mut ft_buf = Vec::new();
    BinaryEncoder.encode_tree(&feature_tree, &mut ft_buf)?;
    let feature_tree_hash = Sha512Hasher.hash_tree_encoded(&ft_buf);
    store.put(&feature_tree_hash, &Object::Tree(feature_tree))?;

    let feature_commit_cmd = CreateCommit {
        tree_hash: feature_tree_hash,
        parents: vec![commit1_hash],
        author: bob.clone(),
        committer: bob.clone(),
        message: "Work on feature".into(),
        encoder: Box::new(BinaryEncoder),
        hasher: Box::new(Sha512Hasher),
    };
    let feature_commit_hash = feature_commit_cmd.execute(&mut store, &mut refs)?;
    println!("Commit on feature branch: {}", feature_commit_hash);

    SetHead {
        target: "refs/heads/main".to_string(),
    }
    .execute(&mut store, &mut refs)?;
    let main_bytes = b"Main line work";
    let main_blob = Blob::new(main_bytes.to_vec());
    let main_blob_hash = Sha512Hasher.hash_blob(main_bytes);
    store.put(&main_blob_hash, &Object::Blob(main_blob))?;
    let main_entry = TreeEntry::new("main.txt".to_string(), EntryKind::Blob, main_blob_hash)
        .map_err(VctrlError::Tree)?;
    let main_tree = Tree::new(vec![main_entry]).map_err(VctrlError::Tree)?;
    let mut mt_buf = Vec::new();
    BinaryEncoder.encode_tree(&main_tree, &mut mt_buf)?;
    let main_tree_hash = Sha512Hasher.hash_tree_encoded(&mt_buf);
    store.put(&main_tree_hash, &Object::Tree(main_tree))?;
    let main_commit_cmd = CreateCommit {
        tree_hash: main_tree_hash,
        parents: vec![commit1_hash],
        author: alice.clone(),
        committer: alice.clone(),
        message: "Main branch work".into(),
        encoder: Box::new(BinaryEncoder),
        hasher: Box::new(Sha512Hasher),
    };
    let main_commit_hash = main_commit_cmd.execute(&mut store, &mut refs)?;
    println!("Commit on main: {}", main_commit_hash);

    println!("--- 5. Merge feature into main ---");
    let merge_cmd = MergeBranch {
        branch_name: feature_branch.to_string(),
        author: alice.clone(),
        committer: alice.clone(),
        merger: Box::new(ThreeWayMerger),
        resolver: Box::new(OursResolver),
        encoder: Box::new(BinaryEncoder),
        hasher: Box::new(Sha512Hasher),
    };
    let merge_commit_hash = merge_cmd.execute(&mut store, &mut refs)?;
    println!("Merge commit: {}", merge_commit_hash);

    println!("--- 6. Tags ---");
    CreateLightweightTag {
        name: "v0.1.0".to_string(),
        target: commit1_hash,
    }
    .execute(&mut store, &mut refs)?;
    println!("Lightweight tag 'v0.1.0' created");

    let annotated_cmd = CreateAnnotatedTag {
        name: "v0.2.0".to_string(),
        target: merge_commit_hash,
        tagger: alice.clone(),
        message: "Pre-release".to_string(),
        encoder: Box::new(BinaryEncoder),
        hasher: Box::new(Sha512Hasher),
        signer: None,
    };
    let tag_hash = annotated_cmd.execute(&mut store, &mut refs)?;
    println!("Annotated tag 'v0.2.0' created (hash: {})", tag_hash);

    let tags = ListTags.execute(&mut store, &mut refs)?;
    println!("All tags: {:?}", tags);

    println!("--- 7. Checkout ---");
    let checkout_cmd = Checkout {
        tree_hash: tree1_hash,
    };
    let files = checkout_cmd.execute(&mut store, &mut refs)?;
    for (path, data) in &files {
        println!("  {} ({} bytes)", path, data.len());
    }

    println!("--- 8. Diff ---");
    let diff_cmd = DiffCommits {
        old_commit: commit1_hash,
        new_commit: main_commit_hash,
    };
    let diffs = diff_cmd.execute(&mut store, &mut refs)?;
    for d in &diffs {
        match &d.kind {
            DiffKind::Added { .. } => println!("  + {}", d.name),
            DiffKind::Removed => println!("  - {}", d.name),
            DiffKind::Modified { .. } => println!("  M {}", d.name),
        }
    }

    println!("--- 9. Blame ---");
    let blame_cmd = Annotate {
        start_commit: main_commit_hash,
        path: "main.txt".to_string(),
    };
    let blame_entries = blame_cmd.execute(&mut store, &mut refs)?;
    for entry in &blame_entries {
        println!(
            "  commit {} by {}: {}",
            &entry.commit_hash.to_hex()[..8],
            entry.author.name,
            entry.message
        );
    }

    println!("--- 10. Stash ---");
    let wip_bytes = b"Temporary work in progress";
    let wip_blob = Blob::new(wip_bytes.to_vec());
    let wip_hash = Sha512Hasher.hash_blob(wip_bytes);
    store.put(&wip_hash, &Object::Blob(wip_blob))?;
    let wip_entry = TreeEntry::new("wip.txt".to_string(), EntryKind::Blob, wip_hash)
        .map_err(VctrlError::Tree)?;
    let wip_tree = Tree::new(vec![wip_entry]).map_err(VctrlError::Tree)?;
    let mut wip_buf = Vec::new();
    BinaryEncoder.encode_tree(&wip_tree, &mut wip_buf)?;
    let wip_tree_hash = Sha512Hasher.hash_tree_encoded(&wip_buf);
    store.put(&wip_tree_hash, &Object::Tree(wip_tree))?;

    let stash_push_cmd = StashPush {
        tree_hash: wip_tree_hash,
        author: alice.clone(),
        message: Some("Work in progress".to_string()),
        encoder: Box::new(BinaryEncoder),
        hasher: Box::new(Sha512Hasher),
    };
    let stash_hash = stash_push_cmd.execute(&mut store, &mut refs)?;
    println!("Stashed as {}", stash_hash);

    let stash_list = StashList.execute(&mut store, &mut refs)?;
    println!("Stash list has {} entry", stash_list.len());

    let popped = StashPop.execute(&mut store, &mut refs)?;
    println!("Stash popped, tree hash: {:?}", popped);

    println!("--- 11. Rebase ---");
    let test_branch = "refs/heads/rebase-test";
    CreateBranch {
        name: test_branch.to_string(),
        hash: main_commit_hash,
    }
    .execute(&mut store, &mut refs)?;
    SetHead {
        target: test_branch.to_string(),
    }
    .execute(&mut store, &mut refs)?;
    let rebase_cmd = Rebase {
        upstream: commit1_hash,
        onto: feature_commit_hash,
        author: alice.clone(),
        committer: alice.clone(),
        merger: Box::new(ThreeWayMerger),
        resolver: Box::new(OursResolver),
        encoder: Box::new(BinaryEncoder),
        hasher: Box::new(Sha512Hasher),
    };
    let rebase_head = rebase_cmd.execute(&mut store, &mut refs)?;
    println!("Rebase done, new HEAD: {}", rebase_head);

    println!("--- 12. Cherry-pick ---");
    SetHead {
        target: "refs/heads/main".to_string(),
    }
    .execute(&mut store, &mut refs)?;
    let cherry_cmd = CherryPick {
        commit_hash: feature_commit_hash,
        author: bob.clone(),
        committer: bob.clone(),
        merger: Box::new(ThreeWayMerger),
        resolver: Box::new(OursResolver),
        encoder: Box::new(BinaryEncoder),
        hasher: Box::new(Sha512Hasher),
    };
    match cherry_cmd.execute(&mut store, &mut refs) {
        Ok(cherry_hash) => println!("Cherry-pick created: {}", cherry_hash),
        Err(e) => {
            println!("Cherry-pick failed (known issue): {}", e);
        }
    }

    println!("--- 13. Octopus merge ---");
    let branch_a = "refs/heads/octo-a";
    let branch_b = "refs/heads/octo-b";
    let branch_c = "refs/heads/octo-c";

    for &name in &[branch_a, branch_b, branch_c] {
        CreateBranch {
            name: name.to_string(),
            hash: commit1_hash,
        }
        .execute(&mut store, &mut refs)?;
    }

    fn small_commit(
        store: &mut MemoryStore,
        refs: &mut MemoryRefStore,
        branch: &str,
        filename: &str,
        data: &[u8],
        parent_hash: Hash,
        author: UserID,
    ) -> Result<Hash, VctrlError> {
        SetHead {
            target: branch.to_string(),
        }
        .execute(store, refs)?;
        let blob_hash = Sha512Hasher.hash_blob(data);
        store.put(&blob_hash, &Object::Blob(Blob::new(data.to_vec())))?;
        let entry = TreeEntry::new(filename.to_string(), EntryKind::Blob, blob_hash)
            .map_err(VctrlError::Tree)?;
        let tree = Tree::new(vec![entry]).map_err(VctrlError::Tree)?;
        let mut buf = Vec::new();
        BinaryEncoder.encode_tree(&tree, &mut buf)?;
        let tree_hash = Sha512Hasher.hash_tree_encoded(&buf);
        store.put(&tree_hash, &Object::Tree(tree))?;
        CreateCommit {
            tree_hash,
            parents: vec![parent_hash],
            author: author.clone(),
            committer: author,
            message: format!("commit on {}", filename),
            encoder: Box::new(BinaryEncoder),
            hasher: Box::new(Sha512Hasher),
        }
        .execute(store, refs)
    }

    let _a = small_commit(
        &mut store,
        &mut refs,
        branch_a,
        "a.txt",
        b"A",
        commit1_hash,
        alice.clone(),
    )?;
    let _b = small_commit(
        &mut store,
        &mut refs,
        branch_b,
        "b.txt",
        b"B",
        commit1_hash,
        alice.clone(),
    )?;
    let _c = small_commit(
        &mut store,
        &mut refs,
        branch_c,
        "c.txt",
        b"C",
        commit1_hash,
        alice.clone(),
    )?;

    let octopus_cmd = OctopusMerge {
        branch_names: vec![branch_a.to_string(), branch_b.to_string()],
        author: alice.clone(),
        committer: alice.clone(),
        merger: Box::new(ThreeWayMerger),
        resolver: Box::new(OursResolver),
        encoder: Box::new(BinaryEncoder),
        hasher: Box::new(Sha512Hasher),
    };
    let octo_hash = octopus_cmd.execute(&mut store, &mut refs)?;
    println!("Octopus merge result: {}", octo_hash);

    println!("--- 14. Verify commit (dummy) ---");
    let verify_cmd = VerifyCommit {
        commit_hash: commit1_hash,
        verifier: Box::new(DummyVerifier),
        encoder: Box::new(BinaryEncoder),
        hasher: Box::new(Sha512Hasher),
    };
    let verified = verify_cmd.execute(&mut store, &mut refs)?;
    println!("Verification result: {}", verified);

    println!("--- 15. Fsck ---");
    let fsck_cmd = Fsck {
        encoder: Box::new(BinaryEncoder),
        hasher: Box::new(Sha512Hasher),
    };
    let errors = fsck_cmd.execute(&mut store, &mut refs)?;
    if errors.is_empty() {
        println!("All objects are valid.");
    } else {
        println!("Errors found: {:?}", errors);
    }

    println!("--- 16. Garbage collection ---");
    let garbage_bytes = b"This blob is not referenced";
    let garbage_blob = Blob::new(garbage_bytes.to_vec());
    let garbage_hash = Sha512Hasher.hash_blob(garbage_bytes);
    store.put(&garbage_hash, &Object::Blob(garbage_blob))?;
    let removed = gc::gc(&mut store, &refs)?;
    println!("GC removed {} unreachable object(s)", removed);

    println!("--- 17. Show ---");
    let show_cmd = Show {
        commit_hash: commit1_hash,
    };
    let output = show_cmd.execute(&mut store, &mut refs)?;
    println!(
        "commit {}: '{}' by {}",
        &hash_of_commit(&output.commit).to_hex()[..8],
        output.commit.message,
        output.commit.author.name
    );
    if let Some(d) = output.diff {
        println!("  Diff entries: {}", d.len());
    } else {
        println!("  (no diff, initial commit)");
    }

    println!("--- 18. Log graph ---");
    let head_hash = refs.head()?.unwrap();
    let log_graph_cmd = LogGraph { head: head_hash };
    let graph = log_graph_cmd.execute(&mut store, &mut refs)?;
    for node in &graph {
        println!(
            "  {} parent_indices: {:?}  msg: {}",
            &node.hash.to_hex()[..8],
            node.parent_indices,
            node.message
        );
    }

    println!("--- 19. Patch ---");
    let old_tree = Tree::new(vec![
        TreeEntry::new("file.txt".to_string(), EntryKind::Blob, readme_hash)
            .map_err(VctrlError::Tree)?,
    ])
    .map_err(VctrlError::Tree)?;
    let new_data = b"Modified content";
    let new_blob_hash = Sha512Hasher.hash_blob(new_data);
    store.put(&new_blob_hash, &Object::Blob(Blob::new(new_data.to_vec())))?;
    let new_tree = Tree::new(vec![
        TreeEntry::new("file.txt".to_string(), EntryKind::Blob, new_blob_hash)
            .map_err(VctrlError::Tree)?,
    ])
    .map_err(VctrlError::Tree)?;

    let patch_data = generate_patch(&old_tree, &new_tree)?;
    let applied = apply_patch(&old_tree, &patch_data, &mut store, &Sha512Hasher)?;
    assert_eq!(applied.entries()[0].hash, new_blob_hash);
    println!("Patch roundtrip succeeded.");

    println!("--- 20. Describe ---");
    let describe_cmd = Describe {
        commit_hash: merge_commit_hash,
        max_commits_to_search: 20,
    };
    match describe_cmd.execute(&mut store, &mut refs)? {
        Some(desc) => println!("Description: {}", desc),
        None => println!("No matching tag found."),
    }

    println!("--- 21. RevWalk ---");
    let walk = RevWalk::new(&store, &[head_hash])?;
    let all: Vec<_> = walk.collect::<Result<Vec<_>, _>>()?;
    println!("RevWalk returned {} commits", all.len());

    println!("--- 22. Index ---");
    let mut index = Index::new();
    index.add(
        TreeEntry::new("staged.txt".to_string(), EntryKind::Blob, readme_hash)
            .map_err(VctrlError::Tree)?,
    );
    let tree_from_index = index.into_tree()?;
    println!(
        "Index built a tree with {} entry",
        tree_from_index.entries().len()
    );

    println!("--- 23. List branches ---");
    let branches = ListBranches.execute(&mut store, &mut refs)?;
    for (name, hash, active) in &branches {
        println!(
            "  {} -> {}{}",
            name,
            &hash.to_hex()[..8],
            if *active { " (active)" } else { "" }
        );
    }

    println!("--- 24. Delete branch ---");
    DeleteBranch {
        name: test_branch.to_string(),
    }
    .execute(&mut store, &mut refs)?;
    println!("Deleted branch '{}'", test_branch);

    println!("\nAll examples completed successfully.");
    Ok(())
}
