# libvctrl v0.5.0

A robust, content‑addressed version control **engine** for arbitrary data, designed to be embedded into applications.

libvctrl provides the core data model, storage abstractions, hashing, encoding, commands, diffing, three‑way merging, and cryptographic signing needed to build version control functionality directly into applications – without shelling out to an external VCS or depending on a CLI tool. It is a **library only** and does not ship a binary.

---

## Features at a Glance

- **Content‑addressed object model** – Blob, Tree, Commit, Tag
- **Pluggable hashing** – SHA‑512 default, any `Hasher` trait implementation
- **Pluggable encoding** – binary format with versioning, support for custom headers and signatures
- **Full Version Control Operations**
  - Commit creation, checkout, diff, log, blame, stash, merge, rebase, cherry‑pick, revert
  - Octopus merge, fast‑forward merge
  - Branch & tag management (lightweight and annotated, with signing)
- **Storage backends** – in‑memory (`MemoryStore`) and append‑only file (`FileStore`) with tombstone deletion for garbage collection
- **Thread‑safe adapters** – `SyncAdapter` wraps any `ObjectStore`/`RefStore` with `Arc<Mutex<>>`
- **Cryptographic signing & verification** – trait‑based `Signer`/`Verifier` (Ed25519 example included)
- **Advanced merging** – three‑way merge with pluggable conflict resolvers
- **Reflog** – optional log of all reference changes
- **RevWalk** – timestamp‑ordered commit iterator returning `(Hash, Commit)`
- **Patch generation & application** (blob‑only)
- **Garbage collection** – mark‑and‑sweep with tombstone support in file store
- **Extensible** – custom encoders, hashers, merge strategies, transports

---

## Data Model

All objects are identified by a 64‑byte `Hash` (SHA‑512).

| Type        | Description                                                                                                                                                  |
| ----------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `Hash`      | A 64‑byte array, serialized as hex string.                                                                                                                   |
| `Blob`      | Raw byte content.                                                                                                                                            |
| `TreeEntry` | A named entry pointing to a blob or subtree. Fields: `name: String`, `kind: EntryKind` (Blob / Tree), `hash: Hash`.                                          |
| `Tree`      | Immutable sorted list of `TreeEntry`. Built via `Tree::new` which sorts and deduplicates.                                                                    |
| `Commit`    | Snapshot of a tree with metadata. Fields: `tree`, `parents`, `author`, `committer`, `timestamp`, `message`, `signature`, `headers` (custom key‑value pairs). |
| `Tag`       | A named pointer to a commit (lightweight) or annotated tag object. Annotated tags store `tagger`, `message`, `timestamp` and optional `signature`.           |
| `Object`    | Enum `Blob                                                                                                                                                   | Tree | Commit | Tag`. |
| `UserID`    | `name` and `email` strings with length validation (1‑255 chars).                                                                                             |

---

## Hashing & Encoding

### Trait `Hasher`

```rust
pub trait Hasher {
    fn hash_blob(&self, data: &[u8]) -> Hash;
    fn hash_tree_encoded(&self, data: &[u8]) -> Hash;
    fn hash_commit_encoded(&self, data: &[u8]) -> Hash;
    fn hash_tag_encoded(&self, data: &[u8]) -> Hash;
}
```

Default implementation: `Sha512Hasher` – prefixes the data with type and length, then hashes with SHA‑512.

### Trait `Encoder` / `Decoder`

```rust
pub trait Encoder {
    fn encode_tree(&self, tree: &Tree, buf: &mut Vec<u8>) -> Result<(), VctrlError>;
    fn encode_commit(&self, commit: &Commit, buf: &mut Vec<u8>) -> Result<(), VctrlError>;
    fn encode_tag(&self, tag: &Tag, buf: &mut Vec<u8>) -> Result<(), VctrlError>;
}

pub trait Decoder {
    fn decode_tree(&self, data: &[u8]) -> Result<Tree, VctrlError>;
    fn decode_commit(&self, data: &[u8]) -> Result<Commit, VctrlError>;
    fn decode_tag(&self, data: &[u8]) -> Result<Tag, VctrlError>;
}
```

Default: `BinaryEncoder` / `BinaryDecoder` – versioned binary format with length‑delimited fields, supports commit version 2 (with headers) and tag version 2 (with signature). Strict limits protect against malicious large allocations.

### Trait `HashVerifier`

Extension of `Hasher` providing `verify_blob`, `verify_tree_encoded`, `verify_commit_encoded`, `verify_tag_encoded`. Automatically implemented for all `Hasher` implementors.

---

## Storage Abstraction

### `ObjectStore` trait

```rust
pub trait ObjectStore {
    fn put(&mut self, hash: &Hash, obj: &Object) -> Result<(), VctrlError>;
    fn get(&self, hash: &Hash) -> Result<Option<Object>, VctrlError>;
    fn exists(&self, hash: &Hash) -> Result<bool, VctrlError>;
    fn all_hashes(&self) -> Result<Vec<Hash>, VctrlError>;
    fn remove(&mut self, hash: &Hash) -> Result<(), VctrlError>;
}
```

### `RefStore` trait

```rust
pub trait RefStore {
    fn set_ref(&mut self, name: &str, hash: &Hash) -> Result<(), VctrlError>;
    fn get_ref(&self, name: &str) -> Result<Option<Hash>, VctrlError>;
    fn delete_ref(&mut self, name: &str) -> Result<(), VctrlError>;
    fn set_head(&mut self, target: &str) -> Result<(), VctrlError>;
    fn head(&self) -> Result<Option<Hash>, VctrlError>;
    fn head_ref_name(&self) -> Result<Option<String>, VctrlError>;
    fn list_refs(&self, prefix: &str) -> Result<Vec<String>, VctrlError>;
}
```

### `ObjectStoreExt` (extension trait)

Provides convenience methods: `get_commit`, `get_tree`, `get_blob`, `get_verified`. The latter recomputes the hash using a given encoder and hasher and returns `Corrupted` on mismatch.

### In‑Memory Backend

- `MemoryStore` – hash map based.
- `MemoryRefStore` – hash map for refs + optional reflog support.

### File Backend (`FileStore`)

Append‑only binary file with magic header. Supports:

- Object records (blob, tree, commit, tag)
- Ref operations (set_ref, delete_ref, set_head)
- Tombstone deletion (`REC_DEL_OBJECT`) – deleted objects are logically removed and remain marked across restarts. The file grows over time; compaction is not yet implemented.

### Thread‑Safe Wrapper

`SyncAdapter<S>` wraps an `ObjectStore` or `RefStore` inside `Arc<Mutex<S>>` and implements `SyncObjectStore` / `SyncRefStore` with `&self` methods. Mutex poisoning returns `Backend` error instead of panicking.

---

## Cryptography

```rust
pub trait Signer: Send + Sync {
    fn sign(&self, data: &[u8]) -> Result<Vec<u8>, VctrlError>;
}

pub trait Verifier: Send + Sync {
    fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, VctrlError>;
}
```

Provided example: Ed25519 via `ed25519‑dalek` (not part of the core library, but used in tests).

---

## Diff & Merge

### Tree Diff

```rust
pub trait TreeDiff {
    fn diff(&self, old_tree: &Tree, new_tree: &Tree) -> Result<Vec<DiffEntry>, VctrlError>;
}
```

`TreeDiffer` compares trees and returns a list of `DiffEntry` with kind `Added`, `Removed`, or `Modified`.

### Three‑Way Merge

```rust
pub trait ThreeWayMerge {
    fn merge(
        &self,
        store: &mut dyn ObjectStore,
        base: &Hash,
        ours: &Hash,
        theirs: &Hash,
        resolver: &dyn ConflictResolver,
        encoder: &dyn Encoder,
        hasher: &dyn Hasher,
    ) -> Result<Hash, VctrlError>;
}
```

`ThreeWayMerger` recursively merges trees. For conflicting blobs, it calls the `ConflictResolver` trait.

```rust
pub trait ConflictResolver {
    fn resolve(&self, base: &[u8], ours: &[u8], theirs: &[u8]) -> Option<Vec<u8>>;
}
```

### Merge Base

- `find_merge_base(store, a, b)` – BFS with a limit (100k) to find a common ancestor.
- `is_ancestor(store, ancestor, descendant)` – shortcut using `find_merge_base`.

### Merge Strategies (future extension)

A `MergeStrategy` trait is defined for pluggable strategies, currently unused by built‑in commands.

---

## Reflog

```rust
pub struct ReflogEntry {
    pub ref_name: String,
    pub old_hash: Option<Hash>,
    pub new_hash: Hash,
    pub timestamp: DateTime<Utc>,
    pub message: String,
}

pub trait ReflogStore {
    fn log_ref_update(&mut self, ref_name: &str, old_hash: Option<Hash>, new_hash: Hash, message: &str) -> Result<(), VctrlError>;
    fn reflog(&self, ref_name: &str) -> Result<Vec<ReflogEntry>, VctrlError>;
}
```

`MemoryRefStore` implements `ReflogStore`, automatically logging every `set_ref` call.

---

## Index (Staging Area)

```rust
pub struct Index {
    entries: BTreeMap<String, TreeEntry>,
}
```

Provides `add`, `remove`, `get`, `iter`, `to_tree()`, `into_tree()`, `clear`. Can be used to build a tree incrementally before committing.

---

## RevWalk

```rust
pub struct RevWalk<'a> { ... }

impl<'a> Iterator for RevWalk<'a> {
    type Item = Result<(Hash, Commit), VctrlError>;
}
```

Returns commits in **timestamp‑descending** order (like `git log --date-order`). Accepts multiple starting tips. Uses a binary heap and visited set.

---

## Patch

- `generate_patch(old_tree, new_tree)` – creates a versioned binary patch (blob‑only; returns error if tree entries are involved).
- `apply_patch(base_tree, patch_data, store, hasher)` – applies a patch to a base tree, producing a new tree.

The `DiffPatch` and `ApplyPatch` commands make these accessible as `Command` implementations.

---

## Command Trait

```rust
pub trait Command {
    type Output;
    fn execute(
        &self,
        store: &mut dyn ObjectStore,
        refs: &mut dyn RefStore,
    ) -> Result<Self::Output, VctrlError>;
}
```

Every operation is a struct implementing this trait. The caller provides the storage and reference backend. All commands are synchronous.

### List of Commands

| Command                | Struct                                                                               | Output                                                | Description                                                                                                                                      |
| ---------------------- | ------------------------------------------------------------------------------------ | ----------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| `Init`                 | `author`, `encoder`, `hasher`                                                        | `Hash` (initial commit)                               | Creates an empty tree and initial commit, sets `refs/heads/main` and HEAD.                                                                       |
| `CreateCommit`         | `tree_hash`, `parents`, `author`, `committer`, `message`, `encoder`, `hasher`        | `Hash` (commit)                                       | Creates a commit object and updates current branch.                                                                                              |
| `Checkout`             | `tree_hash`                                                                          | `Vec<(String, Vec<u8>)>`                              | Recursively reads a tree and returns all files as (path, content).                                                                               |
| `Log`                  | –                                                                                    | `Vec<Commit>`                                         | Returns first‑parent linear history from HEAD.                                                                                                   |
| `LogGraph`             | `head`                                                                               | `Vec<GraphCommit>`                                    | Returns commit history with parent indices for graph drawing. `GraphCommit` includes `hash`, `message`, `author`, `timestamp`, `parent_indices`. |
| `DiffCommits`          | `old_commit`, `new_commit`                                                           | `Vec<DiffEntry>`                                      | Computes tree diff between two commits.                                                                                                          |
| `DiffPatch`            | `old_tree_hash`, `new_tree_hash`                                                     | `Vec<u8>`                                             | Generates a binary patch between two trees.                                                                                                      |
| `ApplyPatch`           | `base_tree_hash`, `patch_data`, `encoder`, `hasher`                                  | `Hash` (new tree)                                     | Applies a patch to a base tree.                                                                                                                  |
| `CherryPick`           | `commit_hash`, `author`, `committer`, `merger`, `resolver`, `encoder`, `hasher`      | `Hash` (new commit)                                   | Cherry‑picks a single commit onto HEAD.                                                                                                          |
| `Revert`               | `commit_hash`, `author`, `committer`, `encoder`, `hasher`                            | `Hash` (new revert commit)                            | Reverts the changes introduced by a commit.                                                                                                      |
| `MergeCommand`         | `base`, `ours`, `theirs`, `merger`, `resolver`, `encoder`, `hasher`                  | `Hash` (merged tree)                                  | Performs a three‑way tree merge (no commit).                                                                                                     |
| `MergeBranch`          | `branch_name`, `author`, `committer`, `merger`, `resolver`, `encoder`, `hasher`      | `Hash` (merge commit)                                 | Merges the given branch into HEAD. Auto‑detects fast‑forward.                                                                                    |
| `OctopusMerge`         | `branch_names`, `author`, `committer`, `merger`, `resolver`, `encoder`, `hasher`     | `Hash` (merge commit)                                 | Merges multiple branches sequentially onto HEAD.                                                                                                 |
| `Rebase`               | `upstream`, `onto`, `author`, `committer`, `merger`, `resolver`, `encoder`, `hasher` | `Hash` (new HEAD)                                     | Replays commits from HEAD onto `onto`, skipping `upstream`.                                                                                      |
| `CreateBranch`         | `name`, `hash`                                                                       | `()`                                                  | Creates a branch reference (must start with `refs/heads/`).                                                                                      |
| `DeleteBranch`         | `name`                                                                               | `()`                                                  | Deletes a branch reference.                                                                                                                      |
| `GetBranch`            | `name`                                                                               | `Option<Hash>`                                        | Reads the hash pointed by a branch.                                                                                                              |
| `SetHead`              | `target`                                                                             | `()`                                                  | Sets HEAD to a branch name (validated) or a direct commit hash.                                                                                  |
| `ListBranches`         | –                                                                                    | `Vec<(String, Hash, bool)>`                           | Lists all branches with their tip hash and whether they are active.                                                                              |
| `CreateLightweightTag` | `name`, `target`                                                                     | `()`                                                  | Creates a lightweight tag pointing directly to a commit.                                                                                         |
| `CreateAnnotatedTag`   | `name`, `target`, `tagger`, `message`, `encoder`, `hasher`, `signer` (optional)      | `Hash` (tag object)                                   | Creates an annotated tag, optionally signed.                                                                                                     |
| `DeleteTag`            | `name`                                                                               | `()`                                                  | Deletes a tag.                                                                                                                                   |
| `ListTags`             | –                                                                                    | `Vec<String>`                                         | Lists all tag names.                                                                                                                             |
| `VerifyCommit`         | `commit_hash`, `verifier: Box<dyn Verifier>`, `encoder`, `hasher`                    | `bool`                                                | Verifies the cryptographic signature of a commit.                                                                                                |
| `VerifyTag`            | `tag_hash`, `verifier`, `encoder`, `hasher`                                          | `bool`                                                | Verifies the cryptographic signature of an annotated tag.                                                                                        |
| `Describe`             | `commit_hash`, `max_commits_to_search` (capped at 100k)                              | `Option<String>`                                      | Finds the nearest tag reachable from the commit and returns a “`tag-N-gxxxxxxx`” string.                                                         |
| `StashPush`            | `tree_hash`, `author`, `message`, `encoder`, `hasher`                                | `Hash` (stash commit)                                 | Saves a tree as a stash, identified by a nanosecond timestamp.                                                                                   |
| `StashPop`             | –                                                                                    | `Option<Hash>` (tree hash)                            | Pops the most recent stash and returns its tree.                                                                                                 |
| `StashList`            | –                                                                                    | `Vec<(String, Hash)>`                                 | Lists all stash entries (ref name and commit hash).                                                                                              |
| `Annotate` (Blame)     | `start_commit`, `path`                                                               | `Vec<BlameEntry>`                                     | Traces the history of a file, returning commit where blob hash changed, limited to 100k commits.                                                 |
| `Show`                 | `commit_hash`                                                                        | `ShowOutput { commit, diff: Option<Vec<DiffEntry>> }` | Shows a commit and its diff to the first parent.                                                                                                 |
| `Fsck`                 | `encoder`, `hasher`                                                                  | `Vec<VctrlError>`                                     | Validates integrity of all objects in the store.                                                                                                 |

---

## Garbage Collection

- `mark_reachable(store, refs)` – returns the set of all objects reachable from any ref or HEAD.
- `gc(store, refs)` – calls `mark_reachable`, then removes all objects not in that set (using `all_hashes` and `remove`). In `FileStore`, removal writes a tombstone record so that objects remain deleted across restarts. Returns the number of removed objects.

---

## Transport

```rust
pub trait Transport {
    fn fetch(&mut self, store: &mut dyn ObjectStore, want: &[Hash]) -> Result<(), VctrlError>;
    fn push(&mut self, store: &dyn ObjectStore, refs: &dyn RefStore, ref_names: &[String]) -> Result<(), VctrlError>;
}
```

Trait for synchronising objects and references between repositories. No built‑in implementations; can be implemented for HTTP, gRPC, etc.

---

## Error Handling

All fallible operations return `Result<T, VctrlError>`.

```rust
pub enum VctrlError {
    Hash(#[from] HashError),
    Tree(#[from] TreeError),
    NotFound(String),
    InvalidRef(String),
    MergeConflict { entry: String, reason: String },
    Io(#[from] std::io::Error),
    Serialization(String),
    Backend(String),
    Other(String),
    Corrupted(String),
    Unsupported(String),
}
```

The library does not panic – all operations propagate errors.

---

## Usage Example (In‑Memory)

```rust
use libvctrl::*;

let mut store = MemoryStore::new();
let mut refs = MemoryRefStore::new();

// Init repository
let init = Init {
    author: UserID::new("Alice".into(), "alice@example.com".into())?,
    encoder: Box::new(BinaryEncoder),
    hasher: Box::new(Sha512Hasher),
};
let root_commit = init.execute(&mut store, &mut refs)?;

// Create a blob
let blob_data = b"Hello, world!";
let blob = Blob::new(blob_data.to_vec());
let blob_hash = Sha512Hasher.hash_blob(blob_data);
store.put(&blob_hash, &Object::Blob(blob))?;

// Build a tree
let entry = TreeEntry::new("README.md".into(), EntryKind::Blob, blob_hash)?;
let tree = Tree::new(vec![entry])?;
let mut buf = Vec::new();
BinaryEncoder.encode_tree(&tree, &mut buf)?;
let tree_hash = Sha512Hasher.hash_tree_encoded(&buf);
store.put(&tree_hash, &Object::Tree(tree))?;

// Create a commit
let commit_cmd = CreateCommit {
    tree_hash,
    parents: vec![root_commit],
    author: UserID::new("Alice".into(), "alice@example.com".into())?,
    committer: UserID::new("Alice".into(), "alice@example.com".into())?,
    message: "Add README".into(),
    encoder: Box::new(BinaryEncoder),
    hasher: Box::new(Sha512Hasher),
};
let new_commit = commit_cmd.execute(&mut store, &mut refs)?;
println!("New commit: {}", new_commit);
```

---

## Minimum Supported Rust Version

1.85.0 (stable)

---

## License

Licensed under the [MIT license](LICENSE).
