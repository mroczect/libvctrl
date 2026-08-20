#![allow(missing_docs)]

use core::hint::black_box;
use core::str::FromStr;

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use libvctrl_handler::{
    Blob, Commit, EntryKind, HASH_LENGTH, Hash, Tree, TreeEntry, UserID, validate_ref_name,
};

fn build_tree_entries(count: usize) -> Vec<TreeEntry> {
    let hash = Hash::from([0_u8; HASH_LENGTH]);
    let mut entries = Vec::with_capacity(count);
    for i in 0..count {
        let name = format!("file_{i:06}");
        if let Ok(entry) = TreeEntry::new(name, EntryKind::Blob, hash) {
            entries.push(entry);
        }
    }
    entries
}

fn bench_tree_build(c: &mut Criterion) {
    let entries = build_tree_entries(5_000);
    let _ = c.bench_function("tree/build_5000_entries", |b| {
        b.iter_batched(
            || entries.clone(),
            |entries| {
                let _ = black_box(Tree::new(entries));
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_validate_refs(c: &mut Criterion) {
    let valid_refs = [
        "refs/heads/main",
        "refs/tags/v1.0.0",
        "refs/remotes/origin/feature/foo",
        "refs/heads/bar",
        "refs/heads/a-branch.name",
    ];
    let invalid_refs = [
        "refs/heads/.hidden",
        "refs/heads/foo.lock/bar",
        "@",
        "refs/heads//double",
    ];

    let _ = c.bench_function("validation/ref_name_valid", |b| {
        b.iter(|| {
            for name in &valid_refs {
                let _ = black_box(validate_ref_name(name));
            }
        });
    });

    let _ = c.bench_function("validation/ref_name_invalid", |b| {
        b.iter(|| {
            for name in &invalid_refs {
                let _ = black_box(validate_ref_name(name));
            }
        });
    });
}

fn bench_hash_parse(c: &mut Criterion) {
    let hex_str = "ab".repeat(HASH_LENGTH); // 64 byte hex = 128 char
    let _ = c.bench_function("hash/from_hex_string", |b| {
        b.iter(|| {
            let _ = black_box(Hash::from_str(&hex_str));
        });
    });
}

fn bench_blob_new(c: &mut Criterion) {
    let data = vec![0x42_u8; 1024 * 1024]; // 1 MiB
    let _ = c.bench_function("blob/new_1MiB", |b| {
        b.iter_batched(
            || data.clone(),
            |data| {
                let _ = black_box(Blob::new(data));
            },
            BatchSize::LargeInput,
        );
    });
}

fn build_user() -> Option<UserID> {
    UserID::new("Bench User".into(), "bench@example.com".into()).ok()
}

fn bench_commit_build(c: &mut Criterion) {
    let Some(user) = build_user() else {
        return;
    };
    let tree_hash = Hash::from([0_u8; HASH_LENGTH]);
    let parents: Vec<Hash> = (0..10).map(|_| Hash::from([1_u8; HASH_LENGTH])).collect();
    let message = "benchmark commit".to_string();

    let _ = c.bench_function("commit/new_10_parents", |b| {
        b.iter_batched(
            || {
                (
                    tree_hash,
                    parents.clone(),
                    user.clone(),
                    user.clone(),
                    message.clone(),
                )
            },
            |(tree, parents, author, committer, msg)| {
                let _ = black_box(Commit::new(tree, parents, author, committer, msg));
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(
    benches,
    bench_tree_build,
    bench_validate_refs,
    bench_hash_parse,
    bench_blob_new,
    bench_commit_build
);
criterion_main!(benches);
