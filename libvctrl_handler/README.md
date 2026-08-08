# `libvctrl_handler`

[![Crates.io](https://img.shields.io/crates/v/libvctrl_handler.svg)](https://crates.io/crates/libvctrl_handler)
[![Docs.rs](https://docs.rs/libvctrl_handler/badge.svg)](https://docs.rs/libvctrl_handler)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](./LICENSE)

**The Unshakeable Contract** – fundamental traits, types, errors, and constants for building a version control system.  
**No implementations. No defaults. Just the constitution.**

---

## Philosophy

- **Mechanism, not policy** – no assumptions about branches, workflows, or storage.
- **Unbounded flexibility, high discipline** – everything is generic and replaceable, but every input is strictly validated.
- **Single source of truth** – all fundamental contracts live exclusively in this crate.
- **Zero dependencies** – only the Rust standard library.

---

## What’s inside?

### Traits (contracts)

| Trait         | Purpose                                 |
| ------------- | --------------------------------------- |
| `ObjectStore` | Content‑addressable object storage      |
| `RefStore`    | Named references (branches, tags)       |
| `Hasher`      | Cryptographic hash function             |
| `Encoder`     | Serialize objects into bytes            |
| `Decoder`     | Deserialize objects from bytes          |
| `Signer`      | Digital signature provider              |
| `Verifier`    | Digital signature verifier              |
| `Transport`   | Fetch/push objects between repositories |

### Types (validated by construction)

| Type        | Description                                             |
| ----------- | ------------------------------------------------------- |
| `Hash`      | 64‑byte SHA‑512 hash (enforced length)                  |
| `Blob`      | Raw file content                                        |
| `Tree`      | Directory listing (sorted, unique entries)              |
| `TreeEntry` | Single entry inside a tree                              |
| `Commit`    | Snapshot with tree, parents, author, committer, message |
| `Tag`       | Named pointer (usually to a commit)                     |
| `UserID`    | Author/committer identity                               |

All fields are **private**; instances can only be created through validated constructors (e.g. `TreeEntry::new`).  
Once created, an instance is guaranteed to be valid.

### Enums

- `EntryKind` – `Blob` or `Tree` (`#[non_exhaustive]` for forward compatibility)
- `VctrlError` – exhaustive error type with `Display`, `Error`, and documentation for every variant

### Constants

- `HASH_LENGTH = 64`
- `MAX_NAME_LENGTH = 255`

### Macros

- **`vctrl_error_other!`** – convenience macro for ad‑hoc `VctrlError::Other` messages.  
  Example (not executed as a doc‑test):
  ```rust,ignore
  vctrl_error_other!("something went wrong: {}", 42);
  ```
  Equivalent to `VctrlError::Other(format!(...))`.

---

## Quick example

```rust
use libvctrl_handler::*;

// Create a hash from known bytes
let hash = Hash::from_bytes(&[0xAB; HASH_LENGTH]).unwrap();

// Build a tree entry (validates name length, non‑empty, etc.)
let entry = TreeEntry::new(
    "hello.txt".into(),
    EntryKind::Blob,
    hash,
).unwrap();

// Build a tree (validates ordering and uniqueness)
let tree = Tree::new(vec![entry]).unwrap();

// Create a user identity
let user = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();

// Create a commit
let commit = Commit::new(
    hash,          // tree
    vec![],        // parents (empty = initial commit)
    user.clone(),  // author
    user,          // committer
    "Initial commit".into(),
);

// Inspect
println!("{}", commit.message()); // "Initial commit"
```

---

## Full API Reference

The complete API documentation with pre/postconditions, invariants, and implementation notes is available on **docs.rs**:  
👉 [https://docs.rs/libvctrl_handler](https://docs.rs/libvctrl_handler)

Below is a concise summary of every public item.

### Traits

- **`ObjectStore`**  
  `put`, `get`, `delete`, `exists` – content‑addressable storage.  
  `exists` is fallible (`Result<bool, VctrlError>`).

- **`RefStore`**  
  `set_ref`, `get_ref`, `delete_ref`, `list_refs` – symbolic name → hash mapping.

- **`Hasher`**  
  `hash(&self, data: &[u8]) -> Hash` – must return exactly `HASH_LENGTH` bytes.

- **`Encoder`**  
  `encode_blob`, `encode_tree`, `encode_commit`, `encode_tag` – round‑trippable with `Decoder`.

- **`Decoder`**  
  `decode_blob`, `decode_tree`, `decode_commit`, `decode_tag` – reconstruct objects from bytes.

- **`Signer`**  
  `sign(&self, data: &[u8]) -> Result<Vec<u8>, VctrlError>`.

- **`Verifier`**  
  `verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, VctrlError>`.

- **`Transport`**  
  `fetch_object`, `push_object` – raw byte transfer between repositories.

### Types

- **`Hash`**  
  `const fn from_bytes(bytes: &[u8]) -> Result<Self, VctrlError>`  
  `const fn as_bytes(&self) -> &[u8; HASH_LENGTH]`  
  Implements `Display` (full hex), `Debug` (short), `Clone`, `Copy`, `Eq`, `Ord`, `Hash`.

- **`Blob`**  
  `fn new(data: Vec<u8>) -> Self`  
  `fn data(&self) -> &[u8]`

- **`Tree`**  
  `fn new(entries: Vec<TreeEntry>) -> Result<Self, VctrlError>`  
  `fn entries(&self) -> &[TreeEntry]`  
  Entries are guaranteed sorted by name, no duplicates.

- **`TreeEntry`**  
  `fn new(name: String, kind: EntryKind, hash: Hash) -> Result<Self, VctrlError>`  
  `fn name(&self) -> &str`, `fn kind(&self) -> EntryKind`, `fn hash(&self) -> &Hash`

- **`Commit`**  
  `fn new(tree: Hash, parents: Vec<Hash>, author: UserID, committer: UserID, message: String) -> Self`  
  `fn tree(&self) -> &Hash`, `fn parents(&self) -> &[Hash]`, `fn author(&self) -> &UserID`, `fn committer(&self) -> &UserID`, `fn message(&self) -> &str`

- **`Tag`**  
  `fn new(name: String, target: Hash, tagger: Option<UserID>, message: String) -> Result<Self, VctrlError>`  
  `fn name(&self) -> &str`, `fn target(&self) -> &Hash`, `fn tagger(&self) -> Option<&UserID>`, `fn message(&self) -> &str`

- **`UserID`**  
  `fn new(name: String, email: String) -> Result<Self, VctrlError>`  
  `fn name(&self) -> &str`, `fn email(&self) -> &str`

### Enums

- **`EntryKind`** (non‑exhaustive)  
  `Blob`, `Tree`

- **`VctrlError`** (non‑exhaustive)  
  Variants: `InvalidHashLength(usize)`, `InvalidName(String)`, `ObjectNotFound(Hash)`, `RefNotFound(String)`, `CorruptedData(String)`, `IoError(String)`, `SerializationError(String)`, `Other(String)`.  
  Implements `Display`, `Error`, `Clone`, `PartialEq`.

### Constants

- **`HASH_LENGTH: usize = 64`**
- **`MAX_NAME_LENGTH: usize = 255`**

### Macros

- **`vctrl_error_other!`**
  ```rust,ignore
  vctrl_error_other!("something went wrong: {}", 42);
  ```
  Expands to `VctrlError::Other(format!(...))`.

---

## License

MIT – see [LICENSE](./LICENSE) for details.
