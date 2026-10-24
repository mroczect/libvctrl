# libvctrl_monumentum

Pluggable `ObjectStore` + `RefStore` backend for libvctrl, backed by monumentum.

```rust
let store = MonumentumStore::new(FileStorage::open("repo.db", 64)?);
let refs = store.clone();
let repo = Repository::new(Box::new(store), Box::new(refs));
```
