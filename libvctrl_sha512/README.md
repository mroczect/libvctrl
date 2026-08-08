# libvctrl_sha512

**A zero-dependency, `no_std` Rust implementation of SHA‑512, HMAC‑SHA‑512, HKDF‑SHA‑512, and optional SHA‑384, HMAC‑SHA‑384, HKDF‑SHA‑384.**

[![Crates.io](https://img.shields.io/crates/v/libvctrl_sha512.svg)](https://crates.io/crates/libvctrl_sha512)
[![Documentation](https://docs.rs/libvctrl_sha512/badge.svg)](https://docs.rs/libvctrl_sha512)
[![License](https://img.shields.io/badge/license-ISC-blue.svg)](LICENSE)

---

## Table of Contents

1. [Overview](#overview)
2. [Features](#features)
3. [Installation](#installation)
4. [Usage Examples](#usage-examples)
   - [SHA‑512](#sha‑512)
   - [HMAC‑SHA‑512](#hmac‑sha‑512)
   - [HKDF‑SHA‑512](#hkdf‑sha‑512)
   - [SHA‑384 & friends](#sha‑384--friends)
5. [API Reference](#api-reference)
   - [Root re‑exports](#root‑re‑exports)
   - [`utils`](#module-utils)
   - [`sha512`](#module-sha512)
   - [`hmac`](#module-hmac)
   - [`hkdf`](#module-hkdf)
   - [`sha384` (feature‑gated)](#module-sha384-feature‑gated)
6. [Security Considerations](#security-considerations)
7. [Performance & Benchmarks](#performance--benchmarks)
8. [Feature Flags](#feature-flags)
9. [License](#license)
10. [Acknowledgements](#acknowledgements)

---

## Overview

`libvctrl_sha512` is a pure‑Rust cryptographic library that implements the **SHA‑512** hash function, **HMAC‑SHA‑512** message authentication code, and **HKDF‑SHA‑512** key derivation function as specified in:

- **FIPS 180‑4** – Secure Hash Standard
- **RFC 2104** – HMAC: Keyed‑Hashing for Message Authentication
- **RFC 5869** – HMAC‑based Extract‑and‑Expand Key Derivation Function (HKDF)

Optionally, with the `sha384` feature (enabled by default), the library also provides **SHA‑384**, **HMAC‑SHA‑384**, and **HKDF‑SHA‑384**, which share the same compression function but with a different initial vector and a truncated output of 48 bytes.

The crate is:

- **`#![no_std]`** – runs without the standard library, ideal for embedded systems, kernels, and WebAssembly.
- **Zero external dependencies** – only the `core` crate is used, minimising supply‑chain risks.
- **Audited and corrected** – previous audit findings (v0.2.0) have been resolved; FIPS 180‑4 padding is now fully compliant.
- **Constant‑time** – all verification operations use a data‑independent comparison routine, preventing timing side‑channel leakage.
- **Streaming and one‑shot** – both incremental and one‑shot APIs are provided for hashing, HMAC, and HKDF.

The code is a modularised fork of Frank Denis’s excellent [hmac‑sha512](https://github.com/jedisct1/rust-hmac-sha512) crate. The core cryptographic logic remains unchanged; this version adds thorough documentation, feature flags, and a cleaner module structure.

---

## Features

- **SHA‑512** – one‑shot, streaming, and constant‑time verification.
- **HMAC‑SHA‑512** – one‑shot, streaming, and constant‑time verification with automatic key hashing for keys longer than the block size (128 bytes).
- **HKDF‑SHA‑512** – extract‑then‑expand key derivation; extracts a 64‑byte PRK, expands to any length ≤ 255 × 64 bytes.
- **SHA‑384 / HMAC‑SHA‑384 / HKDF‑SHA‑384** – activated by the `sha384` feature (default). Uses the same compression function as SHA‑512 but with a different IV and 48‑byte output.
- **`opt_size` feature** – trades some speed for a roughly 75% reduction in code size.
- **Memory zeroisation** – temporary key material is zeroed after use; `Drop` implementation clears padded keys.
- **No unsafe code** except for a single, necessary `core::ptr::read_volatile` in the constant‑time comparison (explained in [Security Considerations](#security-considerations)).
- **Fully documented** – every public item has a doc‑comment, and all examples are tested as part of `cargo test`.

---

## Installation

Add `libvctrl_sha512` to your `Cargo.toml`:

```toml
[dependencies]
libvctrl_sha512 = "0.1.0"
```

### Enabling / disabling SHA‑384

The `sha384` feature is **enabled by default**. If you do not need SHA‑384 and want a smaller binary or shorter compile times, disable default features:

```toml
[dependencies]
libvctrl_sha512 = { version = "0.1.0", default-features = false }
```

You can also enable the `opt_size` feature for a size‑optimised build:

```toml
[dependencies]
libvctrl_sha512 = { version = "0.1.0", features = ["opt_size"] }
```

---

## Usage Examples

All examples below are tested as doctests. You can run them with `cargo test --doc`.

### SHA‑512

```rust
use libvctrl_sha512::Hash;

// One‑shot hashing
let digest = Hash::hash(b"hello world");
assert_eq!(digest.len(), 64);

// Streaming hashing
let mut hasher = Hash::new();
hasher.update(b"hello ");
hasher.update(b"world");
assert_eq!(hasher.finalize(), digest);

// Constant‑time verification
let mut verifier = Hash::new();
verifier.update(b"hello world");
assert!(verifier.verify(&digest));
```

### HMAC‑SHA‑512

```rust
use libvctrl_sha512::HMAC;

// One‑shot MAC
let key = b"my-secret-key";
let msg = b"important message";
let mac = HMAC::mac(msg, key);
assert_eq!(mac.len(), 64);

// Streaming MAC
let mut hmac = HMAC::new(key);
hmac.update(b"important ");
hmac.update(b"message");
assert_eq!(hmac.finalize(), mac);

// Constant‑time verification
assert!(HMAC::verify(msg, key, &mac));
```

### HKDF‑SHA‑512

```rust
use libvctrl_sha512::HKDF;

let ikm = b"shared-secret";
let salt = b"random-salt";
let info = b"encryption-key";

// Extract phase – produce a 64‑byte pseudorandom key
let prk = HKDF::extract(salt, ikm);

// Expand phase – derive a 32‑byte AES key
let mut aes_key = [0u8; 32];
HKDF::expand(&mut aes_key, prk, info);
```

Deriving multiple keys from the same PRK:

```rust
let prk = HKDF::extract(b"master-salt", b"master-ikm");

let mut enc_key = [0u8; 32];
let mut mac_key = [0u8; 64];
HKDF::expand(&mut enc_key, prk, b"encryption");
HKDF::expand(&mut mac_key, prk, b"authentication");
```

### SHA‑384 & friends

When the `sha384` feature is active (default), import from `sha384`:

```rust
use libvctrl_sha512::sha384::{Hash, HMAC, HKDF};

// SHA‑384 hashing
let d = Hash::hash(b"data");
assert_eq!(d.len(), 48);

// HMAC‑SHA‑384
let mac = HMAC::mac(b"msg", b"key");
assert_eq!(mac.len(), 48);

// HKDF‑SHA‑384
let prk = HKDF::extract(b"salt", b"ikm");
let mut okm = [0u8; 42];
HKDF::expand(&mut okm, prk, b"info");
```

---

## API Reference

### Root re‑exports

The crate root re‑exports the most commonly used types:

```rust
pub use sha512::Hash;   // SHA‑512 hasher
pub use hmac::HMAC;     // HMAC‑SHA‑512
pub use hkdf::HKDF;     // HKDF‑SHA‑512
pub use utils::{BLOCKBYTES, BYTES};
```

Thus you can write `use libvctrl_sha512::Hash;` directly.

### Module `utils`

Internal helpers, exposed for completeness.

| Item                                                  | Description                                |
| ----------------------------------------------------- | ------------------------------------------ |
| `BLOCKBYTES: usize = 128`                             | SHA‑512 block size.                        |
| `BYTES: usize = 64`                                   | SHA‑512 output size.                       |
| `fn load_be(base: &[u8], offset: usize) -> u64`       | Load a big‑endian `u64` from a byte slice. |
| `fn store_be(base: &mut [u8], offset: usize, x: u64)` | Store a `u64` as big‑endian bytes.         |
| `fn verify(x: &[u8], y: &[u8]) -> bool`               | Constant‑time slice comparison.            |

### Module `sha512`

The SHA‑512 hash function.

#### `Hash`

```rust
pub struct Hash { /* fields hidden */ }
impl Hash {
    pub fn new() -> Self;
    pub fn update<T: AsRef<[u8]>>(&mut self, input: T);
    pub fn finalize(self) -> [u8; 64];
    pub fn hash<T: AsRef<[u8]>>(input: T) -> [u8; 64];
    pub fn verify(self, expected: &[u8; 64]) -> bool;
}
impl Default for Hash { ... }
impl Copy for Hash { ... }
impl Clone for Hash { ... }
```

- **`new()`** – Creates a new SHA‑512 hasher with the standard initialisation vector.
- **`update()`** – Feeds data into the hasher; can be called multiple times.
- **`finalize()`** – Consumes the hasher and returns the 64‑byte digest. Message padding is applied according to FIPS 180‑4 (128‑bit big‑endian length, upper 64 bits zero).
- **`hash()`** – Convenience method for one‑shot hashing.
- **`verify()`** – Finalizes and compares the digest against `expected` in constant time. Returns `true` if they match.

### Module `hmac`

HMAC‑SHA‑512 (RFC 2104).

#### `HMAC`

```rust
pub struct HMAC { /* fields hidden */ }
impl HMAC {
    pub fn mac<T: AsRef<[u8]>, U: AsRef<[u8]>>(input: T, k: U) -> [u8; 64];
    pub fn new(k: impl AsRef<[u8]>) -> Self;
    pub fn update(&mut self, input: impl AsRef<[u8]>);
    pub fn finalize(self) -> [u8; 64];
    pub fn finalize_verify(self, expected: &[u8; 64]) -> bool;
    pub fn verify<T: AsRef<[u8]>, U: AsRef<[u8]>>(input: T, k: U, expected: &[u8; 64]) -> bool;
}
impl Drop for HMAC { ... }
```

- **`mac()`** – One‑shot HMAC‑SHA‑512 computation. Keys longer than 128 bytes are hashed first.
- **`new()`** – Creates a streaming HMAC context. The key is processed immediately (hashed if necessary and XORed with the inner pad).
- **`update()`** – Feeds additional data.
- **`finalize()`** – Completes the HMAC and returns the 64‑byte tag. Consumes the context.
- **`finalize_verify()`** – Finalizes and compares with `expected` in constant time.
- **`verify()`** – One‑shot verification, equivalent to `mac` + constant‑time compare.

The `Drop` implementation zeroises the padded key buffer.

### Module `hkdf`

HKDF‑SHA‑512 (RFC 5869). The `HKDF` struct is stateless (zero‑sized); all methods are static.

#### `HKDF`

```rust
pub struct HKDF;
impl HKDF {
    pub fn extract(salt: impl AsRef<[u8]>, ikm: impl AsRef<[u8]>) -> [u8; 64];
    pub fn expand(out: &mut [u8], prk: impl AsRef<[u8]>, info: impl AsRef<[u8]>);
}
```

- **`extract()`** – Takes optional salt and input keying material, returns a 64‑byte PRK. This is a single HMAC‑SHA‑512 with the salt as the key and IKM as the message.
- **`expand()`** – Expands the PRK into `out.len()` bytes of output keying material using the `info` string for domain separation.
  - **Panics** if `prk.len()` ≠ 64 or if `out.len() > 255 * 64` (16 320 bytes).

### Module `sha384` (feature‑gated)

Enabled by default. Provides **SHA‑384**, **HMAC‑SHA‑384**, and **HKDF‑SHA‑384**.

The API mirrors that of the SHA‑512 versions exactly, except all outputs are 48 bytes instead of 64.

| Type           | Analogue to    |
| -------------- | -------------- |
| `sha384::Hash` | `sha512::Hash` |
| `sha384::HMAC` | `hmac::HMAC`   |
| `sha384::HKDF` | `hkdf::HKDF`   |

For example:

```rust
let d = sha384::Hash::hash(b"abc");               // [u8; 48]
let mac = sha384::HMAC::mac(b"msg", b"key");       // [u8; 48]
let prk = sha384::HKDF::extract(b"salt", b"ikm");  // [u8; 48]
```

The `HKDF::expand` in SHA‑384 requires a 48‑byte PRK and limits the output to `255 * 48` bytes.

---

## Security Considerations

### Constant‑time verification

All comparisons of digests or MACs are performed using the [`utils::verify`] function. It:

1. Compares lengths first (public information).
2. XORs each byte pair into an accumulator, without branching on the result.
3. On WASM targets, pre‑mixes bytes to avoid potential non‑constant‑time operations in the runtime.
4. Uses a volatile read of the accumulator to prevent compiler optimisations from short‑circuiting the comparison.

This ensures that the time taken by verification does not depend on the number of matching bytes, thus preventing timing side‑channel attacks.

### HMAC key handling

- Keys longer than the block size (128 bytes) are hashed using SHA‑512 (or SHA‑384) before being used.
- After use, temporary key‑derived buffers are zeroed. The `Drop` implementation of `HMAC` clears the padded key buffer.
- Empty keys are allowed but not recommended.

### HKDF recommendations

- **Salt**: Should be random and can be public. Even a fixed salt is better than none.
- **PRK length**: Must be exactly the output length of the underlying hash (64 for SHA‑512, 48 for SHA‑384). Passing an incorrect length panics.
- **Info**: Use distinct `info` strings for different key purposes. The same PRK can safely be used with multiple `info` values.

### `no_std` and unsafe

The crate uses `#![no_std]` and does not allocate memory. The only `unsafe` block is the `core::ptr::read_volatile` in `utils::verify`, which is necessary to prevent the compiler from optimising away the constant‑time loop. The operation is safe in practice because the pointer is valid and the value is not used for memory access.

---

## Performance & Benchmarks

Benchmarks are provided in the `benches/` directory. To run them:

```bash
cargo bench --all-features
```

On a modern x86‑64 processor, hashing 1 KB of data takes a few microseconds. The `opt_size` feature reduces binary size by about 75% (by marking some functions as `inline(never)`) at a cost of roughly 16% lower throughput. For embedded targets with tight flash limits, `opt_size` is recommended.

---

## Feature Flags

| Flag       | Description                                      | Default |
| ---------- | ------------------------------------------------ | ------- |
| `sha384`   | Enables SHA‑384, HMAC‑SHA‑384, and HKDF‑SHA‑384. | Yes     |
| `opt_size` | Optimises for code size at the expense of speed. | No      |

To disable SHA‑384:

```toml
libvctrl_sha512 = { version = "0.1.0", default-features = false }
```

To enable size optimisation:

```toml
libvctrl_sha512 = { version = "0.1.0", features = ["opt_size"] }
```

---

## License

This project is distributed under the **ISC License**.

```
Copyright (c) 2019–2026 Frank Denis
Copyright (c) 2026 mroczect

Permission to use, copy, modify, and/or distribute this software for any
purpose with or without fee is hereby granted, provided that the above
copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
```

---

## Acknowledgements

This crate is a fork of [hmac-sha512](https://github.com/jedisct1/rust-hmac-sha512) by Frank Denis. The original implementation was audited, high‑performance, and minimal. This version adds:

- SHA‑384 support (feature‑gated)
- HKDF (both SHA‑512 and SHA‑384)
- FIPS 180‑4 compliant padding (full 128‑bit length field)
- Comprehensive documentation and doctests
- Criterion benchmarks
- Memory zeroisation improvements

All cryptographic logic is derived from the original work; the core security properties remain unchanged.
