# Documentation Conventions for libvctrl

This guide describes the mandatory conventions for documenting every public
trait, type, function, and module in the `libvctrl` workspace. Following these
guidelines ensures that the API documentation is consistent, comprehensive,
and useful for both maintainers and downstream users.

## General Rules

1. **Use `///` for public items** (traits, types, functions, constants) and  
   `//!` for module-level documentation.
2. **Explain _why_ the item exists**, not just _what_ it does.  
   Describe its purpose, role in the system, and any design rationale.
3. **Prefer active voice and concise sentences.**  
   Example: “Returns the hash of the object” instead of “The hash of the object is returned”.
4. **Use intra-doc links** to reference related items:  
   `[`Type`]`, `[`trait@ObjectStore`]`, `[`module@types`]`.
5. **Never link to private items** from public documentation.  
   If you must mention a private helper, use backticks (`` `private_fn` ``) instead of a link.
6. **Include at least one `# Examples` section** for every public trait and for any type that requires construction or usage explanation.

## Required Sections for Traits

Every public trait must have the following sections in its top-level `///` documentation:

- **`# Purpose`** – one or two sentences about why the trait exists.
- **`# Examples`** – at least one runnable example (` ```rust ` block) that compiles and executes successfully.  
  If the trait is not meant to be implemented directly, show a typical usage via a concrete implementation.
- **`# Errors`** – if any method returns `Result`, enumerate the possible error variants and when they may occur.
- **`# Panics`** – if any method can panic, describe the exact conditions that trigger a panic.  
  If no method panics, state: “This trait does not panic.”

### Example (trait)

````rust
/// Defines the interface for hashing raw data into a fixed-size [`Hash`].
///
/// # Purpose
///
/// The `Hasher` trait abstracts the cryptographic hash algorithm so that
/// downstream code can swap hash implementations without changing core logic.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::{Hash, Hasher, VctrlError};
///
/// struct DummyHasher;
///
/// impl Hasher for DummyHasher {
///     fn hash(&self, _data: &[u8]) -> Result<Hash, VctrlError> {
///         Ok(Hash::from_bytes(&[0u8; 64]).unwrap())
///     }
/// }
///
/// let hasher = DummyHasher;
/// let hash = hasher.hash(b"hello").unwrap();
/// assert_eq!(hash.as_bytes().len(), 64);
/// ```
///
/// # Errors
///
/// - [`VctrlError::InvalidHashLength`] if the underlying hash function
///   returns a digest of unexpected length.
/// - [`VctrlError::IoError`] if the hashing operation fails due to I/O.
///
/// # Panics
///
/// This trait does not panic.
pub trait Hasher {
    fn hash(&self, data: &[u8]) -> Result<Hash, VctrlError>;
}
````

## Required Sections for Types

Every public type must have:

- **A short description** in the first line (before any blank line) that states what the type represents.
- **`# Examples`** section if the type is complex, requires construction logic, or has non-trivial methods.  
  Simple types like `Hash` or `EntryKind` may omit the example if the description is sufficient, but it is encouraged.

### Example (struct)

````rust
/// A content-addressable blob object.
///
/// A `Blob` stores raw file content and is identified by its hash.
/// It is immutable once constructed, which simplifies reasoning about state.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::Blob;
///
/// let blob = Blob::new(b"hello".to_vec());
/// assert_eq!(blob.size(), 5);
/// ```
pub struct Blob {
    data: Vec<u8>,
}
````

## Formatting Guidelines

- Use **proper Markdown**:
  - Code blocks with triple backticks and language tag (` ```rust `).
  - Lists with `-` or `1.`.
  - Headings with `#`, `##`, etc.
- **Intra-doc links** should use `[`...`]` syntax.
  - For a type: `[`Hash`]`.
  - For a trait: `[`trait@ObjectStore`]`.
  - For a module: `[`module@types`]`.
  - For a function: `[`Hash::from_bytes`]`.
- **Avoid redundant explicit targets** like `[`Hash`](crate::Hash)`; simply write `[`Hash`]` if `Hash` is in scope, or use a fully qualified path if needed.
- **Link only to public items.** Private items must be written with backticks without link syntax.

## Template for New Public Items

You can copy and adapt the following template when adding a new trait or type.

````rust
/// Short summary of what this item does.
///
/// # Purpose
///
/// Explain why this item exists and how it fits into the larger system.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::YourType; // adjust import
///
/// // Example code that compiles and runs.
/// let instance = YourType::new(...);
/// assert!(...);
/// ```
///
/// # Errors
///
/// If applicable, list error variants and conditions.
///
/// # Panics
///
/// If applicable, describe panic conditions; otherwise state "This item does not panic."
pub struct YourType { ... }
````

## Enforcement

- All public items **must** follow this guide.
- The CI pipeline runs `cargo test --doc` to ensure all doctests pass.
- `cargo doc` is run to check for broken intra-doc links and missing documentation.
- Use `#![deny(missing_docs)]` in each crate (already present in `libvctrl_handler`) to ensure no public item is left undocumented.
