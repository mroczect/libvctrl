//! Definition of the [`EntryKind`] enum.
//!
//! Logical object type enumerations for `libvctrl_handler`.
//!
//! # Purpose
//!
//! This module defines high-level, discriminative types that categorize the
//! logical kind of an object in the version control system. Rather than
//! exposing raw filesystem mode bits, it provides a semantic enum
//! ([`EntryKind`]) that distinguishes between regular files, executable
//! files, symbolic links, subdirectories, and submodule references.
//!
//! # Design Rationale
//!
//! The enum is kept separate from the low-level mode constants (like those
//! in [`crate::constants::entry_mode`]) to decouple the abstract data model
//! ("what kind of object is this?") from the serialized Unix-style
//! representation ("what permission bits does this object have?"). This
//! allows different backends to map their own mode systems to a uniform set
//! of logical kinds, and makes the core data structures independent of
//! POSIX-specific details.
//!
//! The module itself is deliberately small; it contains only the enum and
//! its documentation. This avoids pulling in dependencies or bloating the
//! crate with logic that belongs to higher-level components (e.g., a decoder
//! implementation).
//!
//! # How to Use This Module
//!
//! The enum is re-exported at both `libvctrl_handler::enums::EntryKind` and
//! `libvctrl_handler::EntryKind`. You can import it from either path:
//!
//! ```
//! use libvctrl_handler::enums::EntryKind;
//! ```
//!
//! or:
//!
//! ```
//! use libvctrl_handler::EntryKind;
//! ```
//!
//! Both refer to the same type.

/// Represents the logical kind of an entry in a version control tree.
///
/// # Purpose
///
/// A [`TreeEntry`] must describe whether it points to
/// regular file content, an executable file, a symbolic link, a
/// subdirectory, or a submodule commit. [`EntryKind`] provides that
/// discrimination without tying the type to specific filesystem permission
/// bits.
///
/// # Design Rationale
///
/// - **`#[non_exhaustive]`**: Ensures that adding new variants in the future
///   (e.g., a hypothetical `GitAttribute` or `Custom`) will not break
///   exhaustive `match` statements in downstream code. External crates must
///   include a wildcard `_ =>` arm.
/// - **`Copy` and `Clone`**: The enum is a lightweight tag (typically 1
///   byte). Making it `Copy` allows it to be passed by value freely, which
///   is essential for a type that appears in many collection lookups and
///   comparisons.
/// - **`Hash` and `Eq`**: Enables entries to be grouped, compared, or used
///   as keys in hash maps, e.g., when indexing trees by entry kind.
/// - **Separation from mode bits**: The mapping from raw mode constants
///   (like `0o100644` or `0o120000`) to [`EntryKind`] is performed by
///   higher-level decoder implementations. This keeps the core crate
///   independent of any particular serialization format.
///
/// # Internal Mechanism
///
/// This is a standard C-like enum. Rust guarantees it occupies the minimum
/// required memory (a single byte on most platforms). No data is attached
/// to any variant, so the size is constant and predictable. The derived
/// [`PartialEq`] and [`Eq`] implementations compare variant tags, which are
/// represented as small integers internally. The derived `Hash`
/// implementation hashes this tag value.
///
/// # Memory Layout
///
/// Because the enum has only five variants, the Rust compiler will typically
/// use a single byte to store the discriminant. This ensures that
/// [`EntryKind`] is as cheap to copy as a `u8`, making it suitable for use
/// in high-frequency tree traversal loops.
///
/// # Relationship to `entry_mode`
///
/// The raw serialized form uses Unix mode bits defined in
/// [`crate::constants::entry_mode`]. Mapping between [`EntryKind`] and those
/// constants is the responsibility of encoder and decoder implementations.
/// This decoupling permits the same logical model to be used with non-Unix
/// backends.
///
/// # Examples
///
/// Basic construction and comparison:
///
/// ```
/// use libvctrl_handler::EntryKind;
///
/// let blob = EntryKind::Blob;
/// let executable = EntryKind::Executable;
/// let symlink = EntryKind::Symlink;
/// let tree = EntryKind::Tree;
/// let submodule = EntryKind::Submodule;
///
/// // File-like kinds are not tree-like
/// assert_ne!(blob, tree);
/// assert_ne!(executable, tree);
/// assert_ne!(symlink, tree);
///
/// // Executable is a distinct variant from Blob
/// assert_ne!(blob, executable);
///
/// // Symlink is not the same as a regular file
/// assert_ne!(blob, symlink);
///
/// // Submodule is its own kind
/// assert_ne!(tree, submodule);
/// ```
///
/// Downstream code must use a wildcard when matching because the enum is
/// `#[non_exhaustive]`:
///
/// ```
/// use libvctrl_handler::EntryKind;
///
/// fn describe(kind: EntryKind) -> &'static str {
///     match kind {
///         EntryKind::Blob => "regular file",
///         EntryKind::Executable => "executable file",
///         EntryKind::Symlink => "symbolic link",
///         EntryKind::Tree => "directory",
///         EntryKind::Submodule => "submodule",
///         _ => "unknown", // required because of #[non_exhaustive]
///     }
/// }
///
/// assert_eq!(describe(EntryKind::Blob), "regular file");
/// assert_eq!(describe(EntryKind::Submodule), "submodule");
/// ```
///
/// Using `EntryKind` as a key in a [`HashMap`](std::collections::HashMap):
///
/// ```
/// use libvctrl_handler::EntryKind;
/// use std::collections::HashMap;
///
/// let mut counts = HashMap::new();
/// counts.insert(EntryKind::Blob, 10);
/// counts.insert(EntryKind::Tree, 2);
///
/// assert_eq!(counts.get(&EntryKind::Blob), Some(&10));
/// assert_eq!(counts.get(&EntryKind::Submodule), None);
/// ```
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EntryKind {
    /// The entry points to a [`Blob`] containing regular
    /// (non-executable) file content.
    ///
    /// This is the default kind for files stored in the version control
    /// system. It corresponds to Unix mode `100644` (regular file,
    /// non-executable) in a typical Git-compatible backend.
    ///
    /// # Design Rationale
    ///
    /// Regular files are the most common entry type in a repository tree.
    /// Keeping this variant simple and without extra metadata ensures that
    /// tree traversal remains cheap.
    ///
    /// # How It Is Represented
    ///
    /// The variant tag is encoded as a single byte by the compiler. The
    /// associated [`Blob`] content is stored separately in the object
    /// database, referenced by a `Hash`(crate::Hash).
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::EntryKind;
    /// assert_eq!(EntryKind::Blob, EntryKind::Blob);
    /// ```
    Blob,

    /// The entry points to a [`Blob`] whose content is marked
    /// as executable.
    ///
    /// On Unix-like systems, this indicates the file should have the
    /// executable permission bit set (e.g., mode `100755`). The underlying
    /// data is still a [`Blob`]; the executable flag is stored at the tree
    /// entry level so that the object store remains agnostic to permissions.
    ///
    /// # Design Rationale
    ///
    /// The distinction between regular and executable files is important for
    /// checkouts, but it does not alter the content-addressing scheme. By
    /// separating the executable bit into the tree entry kind, the same
    /// [`Blob`] can be referenced by both executable and non-executable
    /// entries without duplicating content.
    ///
    /// # How It Is Represented
    ///
    /// This variant uses the same underlying tag representation as
    /// [`EntryKind::Blob`], but with a different discriminant value. It is
    /// still one byte in size.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::EntryKind;
    /// assert_ne!(EntryKind::Executable, EntryKind::Blob);
    /// ```
    Executable,

    /// The entry points to a [`Blob`] representing a symbolic
    /// link.
    ///
    /// The blob content is the target path of the symlink. The version
    /// control system does not follow or interpret the link; it simply
    /// stores and retrieves the target string. On Unix, this corresponds
    /// to mode `120000`.
    ///
    /// # Design Rationale
    ///
    /// Symbolic links require special handling during checkout to recreate
    /// the link rather than writing the target string as a regular file. The
    /// tree entry kind signals this requirement to the checkout implementation.
    ///
    /// # How It Is Represented
    ///
    /// The symlink kind is stored as a separate discriminant. The target
    /// path is stored in a [`Blob`] object, referenced by the entry's
    /// `Hash`(crate::Hash).
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::EntryKind;
    /// assert_ne!(EntryKind::Symlink, EntryKind::Tree);
    /// ```
    Symlink,

    /// The entry points to another [`Tree`] (a subdirectory).
    ///
    /// This is the only entry kind that introduces hierarchy. During
    /// checkout, the tree object referenced by this entry will be
    /// recursively expanded into a directory.
    ///
    /// # Design Rationale
    ///
    /// Trees are the recursive building blocks of a version control system.
    /// By marking an entry as a tree, the system can decide whether to
    /// recursively traverse it or stop at the current level. This enables
    /// efficient sparse checkouts and tree diffing.
    ///
    /// # How It Is Represented
    ///
    /// The variant is stored as a single byte discriminant. The referenced
    /// [`Tree`] is identified by its `Hash`(crate::Hash), which is stored
    /// separately in the [`TreeEntry`].
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::EntryKind;
    /// assert_ne!(EntryKind::Tree, EntryKind::Blob);
    /// ```
    Tree,

    /// The entry references a **submodule** -- a commit in a separate
    /// repository.
    ///
    /// Submodules are identified by a commit hash stored in a special
    /// [`Blob`] (the submodule's HEAD). The tree entry marks
    /// the path as a submodule so that tools can initialise or update the
    /// nested repository accordingly.
    ///
    /// # Design Rationale
    ///
    /// Submodules are represented differently from regular trees because
    /// they point to a commit in another repository rather than a tree
    /// object in the current repository. Keeping them as a distinct variant
    /// allows tools to treat submodule boundaries specially during checkout,
    /// clone, and status operations.
    ///
    /// # How It Is Represented
    ///
    /// The variant is a single byte discriminant. The referenced commit is
    /// stored as a [`Blob`] containing the commit hash, and the entry
    /// carries that blob's `Hash`(crate::Hash).
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::EntryKind;
    /// let kind = EntryKind::Submodule;
    /// assert_eq!(kind, EntryKind::Submodule);
    /// ```
    Submodule,
}
