# libvctrl_sha512

**Zero-dependency, `no_std` implementation of SHA‑512, HMAC‑SHA‑512, HKDF‑SHA‑512, and optional SHA‑384, HMAC‑SHA‑384, HKDF‑SHA‑384.**

[![Crates.io](https://img.shields.io/crates/v/libvctrl_sha512.svg)](https://crates.io/crates/libvctrl_sha512)
[![Documentation](https://docs.rs/libvctrl_sha512/badge.svg)](https://docs.rs/libvctrl_sha512)
[![License](https://img.shields.io/badge/license-ISC-blue.svg)](LICENSE)

---

## Table of Contents

1. [Overview](#overview)
2. [Features](#features)
3. [Installation](#installation)
4. [Usage Examples](#usage-examples)
   - [SHA‑512 Hashing](#sha512-hashing)
   - [HMAC‑SHA512](#hmacsha512)
   - [HKDF‑SHA512](#hdfsha512)
   - [SHA‑384, HMAC‑SHA384, HKDF‑SHA384](#sha384-hmacsha384-hkdfsha384)
5. [API Reference](#api-reference)
   - [Module `utils`](#module-utils)
   - [Module `sha512`](#module-sha512)
   - [Module `hmac`](#module-hmac)
   - [Module `hkdf`](#module-hkdf)
   - [Module `sha384` (feature‑gated)](#module-sha384-featuregated)
6. [Security Considerations](#security-considerations)
7. [Performance](#performance)
8. [Feature Flags](#feature-flags)
9. [License](#license)
10. [Acknowledgements](#acknowledgements)

---

## Overview

`libvctrl_sha512` is a pure‑Rust, self‑contained cryptographic library that implements the SHA‑512 hash function, HMAC‑SHA‑512, and HKDF‑SHA‑512 as specified in FIPS 180‑4, RFC 2104, and RFC 5869. It is designed with **zero external dependencies**, making it ideal for embedded systems, kernels, and other environments where resource usage and supply‑chain trust are critical.

The code is derived from the [hmac-sha512](https://github.com/jedisct1/rust-hmac-sha512) crate by Frank Denis, refactored into modular components while preserving the original performance and security properties. Optional SHA‑384 support is provided via the `sha384` feature.

---

## Features

- **Pure Rust** – no `unsafe` code except for a single volatile read in constant‑time verification (justified and documented).
- **`#![no_std]`** – works in embedded, kernel, and bootloader contexts.
- **Zero external dependencies** – only `core` is used.
- **Constant‑time verification** – all equality checks are performed in O(n) with data‑independent timing.
- **Streaming and one‑shot APIs** – process data incrementally or as a whole.
- **HMAC‑SHA512** – both one‑shot and incremental modes.
- **HKDF‑SHA512 (RFC 5869)** – extract‑then‑expand key derivation.
- **Optional SHA‑384, HMAC‑SHA384, HKDF‑SHA384** – enabled by default.
- **Size optimisation** – `opt_size` feature reduces binary size (~75% smaller) at a moderate performance cost (~16% slower).
- **Criterion benchmarks** – included for performance measurement.

---

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
libvctrl_sha512 = { path = "./libvctrl_sha512" }   # if using locally
# or, from crates.io (once published):
# libvctrl_sha512 = "0.1.0"
```

By default, the `sha384` feature is enabled. To disable it, use:

```toml
[dependencies]
libvctrl_sha512 = { version = "0.1.0", default-features = false }
```

---

## Usage Examples

### SHA‑512 Hashing

```rust
use libvctrl_sha512::Hash;

// One‑shot
let digest = Hash::hash(b"Hello, world!");

// Streaming
let mut hasher = Hash::new();
hasher.update(b"Hello, ");
hasher.update(b"world!");
let digest2 = hasher.finalize();
assert_eq!(digest, digest2);

// Verify in constant time
assert!(Hash::hash(b"data").verify(&Hash::hash(b"data")));
```

### HMAC‑SHA512

```rust
use libvctrl_sha512::HMAC;

// One‑shot
let mac = HMAC::mac(b"message", b"secret-key");

// Streaming
let mut hmac = HMAC::new(b"secret-key");
hmac.update(b"first part ");
hmac.update(b"second part");
let mac2 = hmac.finalize();

// Verify
assert!(HMAC::verify(b"message", b"secret-key", &mac));
```

### HKDF‑SHA512

```rust
use libvctrl_sha512::HKDF;

let ikm = b"shared-secret";
let salt = b"random-salt";
let info = b"session-encryption";

// Extract
let prk = HKDF::extract(salt, ikm);

// Expand – produce a 32‑byte AES‑256 key
let mut okm = [0u8; 32];
HKDF::expand(&mut okm, prk, info);
```

### SHA‑384, HMAC‑SHA384, HKDF‑SHA384

When the `sha384` feature is enabled:

```rust
use libvctrl_sha512::sha384::{Hash, HMAC, HKDF};

// SHA‑384
let digest = Hash::hash(b"Hello, world!");

// HMAC‑SHA384
let mac = HMAC::mac(b"message", b"secret-key");

// HKDF‑SHA384
let prk = HKDF::extract(b"salt", b"ikm");
let mut okm = [0u8; 48];
HKDF::expand(&mut okm, prk, b"info");
```

---

## API Reference

### Module `utils`

Low‑level helpers. Not intended for direct use, but exposed for convenience.

- **Constants**
  - `BLOCKBYTES: usize = 128` – SHA‑512 block size.
  - `BYTES: usize = 64` – SHA‑512 output size.

- **Functions**
  - `load_be(base: &[u8], offset: usize) -> u64` – reads a big‑endian u64.
  - `store_be(base: &mut [u8], offset: usize, x: u64)` – writes a big‑endian u64.
  - `verify(x: &[u8], y: &[u8]) -> bool` – constant‑time equality check.

---

### Module `sha512`

Provides the SHA‑512 hasher.

- **Struct `Hash`**
  - `new() -> Self` – creates a new hasher.
  - `update<T: AsRef<[u8]>>(&mut self, input: T)` – absorbs more data.
  - `finalize(self) -> [u8; 64]` – completes the hash, consumes the instance.
  - `hash<T: AsRef<[u8]>>(input: T) -> [u8; 64]` – one‑shot hash.
  - `verify(self, expected: &[u8; 64]) -> bool` – constant‑time verification.

---

### Module `hmac`

HMAC‑SHA512 implementation.

- **Struct `HMAC`**
  - `mac<T, U>(input: T, k: U) -> [u8; 64]` – one‑shot MAC.
  - `new(k: impl AsRef<[u8]>) -> Self` – creates a streaming HMAC instance.
  - `update(&mut self, input: impl AsRef<[u8]>)` – feeds more data.
  - `finalize(self) -> [u8; 64]` – produces the final MAC.
  - `finalize_verify(self, expected: &[u8; 64]) -> bool` – constant‑time verification.
  - `verify<T, U>(input: T, k: U, expected: &[u8; 64]) -> bool` – one‑shot verification.

---

### Module `hkdf`

HKDF‑SHA512 (RFC 5869).

- **Struct `HKDF`** (stateless)
  - `extract(salt: impl AsRef<[u8]>, ikm: impl AsRef<[u8]>) -> [u8; 64]` – Extract phase, returns PRK.
  - `expand(out: &mut [u8], prk: impl AsRef<[u8]>, info: impl AsRef<[u8]>)` – Expand phase, fills `out` with OKM. Panics if `out.len() > 255 * 64`.

---

### Module `sha384` (feature‑gated)

SHA‑384, HMAC‑SHA384, HKDF‑SHA384. All types mirror their SHA‑512 counterparts, but with 48‑byte outputs.

- **`Hash`**, **`HMAC`**, **`HKDF`** – identical API, output sizes adjusted.

---

## Security Considerations

- **Constant‑time verification**: The `verify` functions in both the hash and HMAC modules use a bitwise‑XOR accumulator with a volatile read to prevent compiler optimisations that would leak timing information. This is the recommended way to compare secret values.
- **Key handling in HMAC**: Keys longer than the block size (128 bytes) are hashed using SHA‑512 before use. This follows the HMAC specification and prevents key‑length attacks.
- **Salt and IKM in HKDF**: The salt should be random and non‑secret for maximum security. The `info` parameter provides domain separation; never reuse the same `info` for different contexts.
- **No `std` dependency**: The crate does not rely on the standard library, which reduces attack surface in trusted execution environments.

---

## Performance

Benchmarks are included in the `benches/` directory. Run them with:

```bash
cargo bench --all-features
```

On a modern x86‑64 CPU, hashing 1 KB of data with SHA‑512 takes a few microseconds. The `opt_size` feature reduces binary size by about 75% at the cost of roughly 16% lower throughput.

---

## Feature Flags

| Flag       | Description                                    | Default  |
| ---------- | ---------------------------------------------- | -------- |
| `sha384`   | Enables SHA‑384, HMAC‑SHA384, and HKDF‑SHA384. | Enabled  |
| `opt_size` | Optimises for binary size (slightly slower).   | Disabled |

---

## License

This crate is distributed under the **ISC License**.

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

This project is a fork and modularisation of the excellent [hmac-sha512](https://github.com/jedisct1/rust-hmac-sha512) crate by [Frank Denis](https://github.com/jedisct1). All cryptographic logic and implementation details originate from his work. The refactoring was done to better integrate with the `libvctrl` workspace and to provide a more structured API.
