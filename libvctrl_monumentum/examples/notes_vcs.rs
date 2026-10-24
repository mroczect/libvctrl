use libvctrl_handler as _;
use monumentum_core as _;
use monumentum_handler as _;

use libvctrl::{FileToCommit, Repository, UserID, VctrlError};
use libvctrl_monumentum::MonumentumStore;
use monumentum_core::store::storage::FileStorage;

fn main() -> Result<(), VctrlError> {
    let db_path = std::env::temp_dir().join("notes_vcs_demo.db");
    let _ = std::fs::remove_file(&db_path);
    let _ = std::fs::remove_file(db_path.with_extension("wal"));

    let engine = FileStorage::open(&db_path, 64)
        .map_err(|e| VctrlError::Other(format!("open storage: {e}")))?;

    let store = MonumentumStore::new(engine);
    let refs = store.clone();
    let mut repo = Repository::new(Box::new(store), Box::new(refs));

    let alice = UserID::new("Alice".into(), "alice@example.com".into())?;

    let c1 = repo.commit(
        "refs/heads/main",
        vec![
            FileToCommit::blob("shopping.md", "# Shopping\n- milk\n"),
            FileToCommit::blob("todo.md", "# Todo\n- learn rust\n"),
        ],
        "initial notes",
        alice.clone(),
    )?;
    println!("[1] committed {}", &c1.to_string()[..12]);

    let c2 = repo.commit(
        "refs/heads/main",
        vec![
            FileToCommit::blob("shopping.md", "# Shopping\n- milk\n- eggs\n"),
            FileToCommit::blob("todo.md", "# Todo\n- learn rust\n- write vcs\n"),
            FileToCommit::blob("ideas.md", "# Ideas\n- build a DB\n"),
        ],
        "added eggs, todo, ideas",
        alice,
    )?;
    println!("[2] committed {}", &c2.to_string()[..12]);

    println!("\nHistory of refs/heads/main:");
    for (hash, commit) in repo.log("refs/heads/main")? {
        println!(
            "  {}  {}  ({} parent)",
            &hash.to_string()[..12],
            commit.message().trim(),
            commit.parents().len()
        );
    }

    println!("\nshopping.md @ commit 1:");
    let old = repo
        .file_at(&c1, "shopping.md")?
        .ok_or_else(|| VctrlError::Other("missing file".into()))?;
    print!("{}", String::from_utf8_lossy(&old));

    println!("\nshopping.md @ commit 2 (HEAD):");
    let current = repo
        .file_at(&c2, "shopping.md")?
        .ok_or_else(|| VctrlError::Other("missing file".into()))?;
    print!("{}", String::from_utf8_lossy(&current));

    println!("\nideas.md @ commit 1 (should not exist):");
    match repo.file_at(&c1, "ideas.md")? {
        Some(_) => println!("  <unexpected: present>"),
        None => println!("  <absent, as expected>"),
    }
    println!("\nideas.md @ commit 2:");
    let ideas = repo
        .file_at(&c2, "ideas.md")?
        .ok_or_else(|| VctrlError::Other("missing".into()))?;
    print!("{}", String::from_utf8_lossy(&ideas));

    let current_tree = repo.snapshot(vec![
        FileToCommit::blob("shopping.md", "# Shopping\n- milk\n- eggs\n"),
        FileToCommit::blob("todo.md", "# Todo\n- learn rust\n- write vcs\n"),
        FileToCommit::blob("ideas.md", "# Ideas\n- build a DB\n"),
    ])?;
    let stored = repo.read_commit(&c2)?;
    println!(
        "\nsnapshot matches HEAD? {}",
        current_tree == *stored.tree()
    );

    println!("\nDatabase file: {}", db_path.display());
    println!("Done.");
    Ok(())
}
