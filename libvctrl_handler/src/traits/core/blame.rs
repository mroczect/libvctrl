//! Blame computation trait.
//!
//! # Architecture
//! This module provides the contracts for attributing lines in a file to specific commits.
//! Blame computation is fundamentally different from standard diffing; it requires traversing
//! history in reverse and tracking line movements across revisions. By isolating this into
//! a dedicated trait, the crate allows consumers to plug in different blame algorithms
//! (e.g., linear history vs. merge-aware) without altering the core engine.
//!
//! # Design Rationale: Immutability and Validation
//! The [`BlameEntry`] struct is constructed via a fallible constructor (`new`). This ensures
//! that invalid states—such as a line range starting at 0 or having a length of 0—cannot
//! exist at runtime. Once constructed, the entry is immutable, guaranteeing that the blame
//! history remains tamper-proof.

use crate::errors::VctrlError;
use crate::types::Hash;

/// A single line range in a file attributed to a commit.
///
/// # Why this exists
/// Represents the atomic unit of blame data. Instead of attributing an entire file to a single
/// commit, Git blame operates on line ranges. This struct encapsulates the mapping between a
/// specific range of lines in a file and the commit that last modified them.
///
/// # How it works
/// The struct holds a reference to the committing [`Hash`], the 1-based line number range,
/// the file path, and an optional commit summary. The `commit_id` is stored as a copied `Hash`
/// (which is a fixed 64-byte array) rather than a reference, to simplify lifetime management
/// when returning vectors of blame entries from background threads.
///
/// # Examples
///
/// ```
/// # use libvctrl_handler::traits::core::blame::BlameEntry;
/// # use libvctrl_handler::Hash;
/// # let hash = Hash::from_bytes(&[0_u8; 64]).unwrap();
/// let entry = BlameEntry::new(
///     hash,
///     10,
///     5,
///     "src/main.rs".to_string(),
///     Some("Initial commit".to_string()),
/// );
/// assert!(entry.is_ok());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlameEntry {
    commit_id: Hash,
    start_line: usize,
    line_count: usize,
    path: String,
    summary: Option<String>,
}

impl BlameEntry {
    /// Creates a new `BlameEntry`.
    ///
    /// # Why this exists
    /// Acts as a validation gate. In text file representations, line numbers are strictly
    /// 1-based and must have a positive length. Allowing a `start_line` of 0 or a
    /// `line_count` of 0 would violate these invariants and cause off-by-one errors
    /// in downstream UI rendering or analysis.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::InvalidBlameRange`] if `start_line` is 0 or `line_count` is 0.
    ///
    /// # Examples
    ///
    /// Valid construction:
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::blame::BlameEntry;
    /// # use libvctrl_handler::Hash;
    /// # let hash = Hash::from_bytes(&[0_u8; 64]).unwrap();
    /// let entry = BlameEntry::new(hash, 1, 10, "file.txt".into(), None);
    /// assert!(entry.is_ok());
    /// ```
    ///
    /// Invalid construction (zero start line):
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::blame::BlameEntry;
    /// # use libvctrl_handler::{Hash, VctrlError};
    /// # let hash = Hash::from_bytes(&[0_u8; 64]).unwrap();
    /// let entry = BlameEntry::new(hash, 0, 10, "file.txt".into(), None);
    /// assert!(matches!(entry, Err(VctrlError::InvalidBlameRange)));
    /// ```
    pub fn new(
        commit_id: Hash,
        start_line: usize,
        line_count: usize,
        path: String,
        summary: Option<String>,
    ) -> Result<Self, VctrlError> {
        if start_line == 0 || line_count == 0 {
            return Err(VctrlError::InvalidBlameRange);
        }
        Ok(Self {
            commit_id,
            start_line,
            line_count,
            path,
            summary,
        })
    }

    /// Returns the commit that last modified these lines.
    ///
    /// # How it works
    /// Because [`Hash`] is a `Copy` type (a fixed-size array wrapper), this accessor returns
    /// a copy rather than a reference. This eliminates the need for lifetime annotations
    /// on the returned value, making it easier to pass the hash to asynchronous tasks or
    /// store in independent data structures.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::blame::BlameEntry;
    /// # use libvctrl_handler::Hash;
    /// # let hash = Hash::from_bytes(&[0_u8; 64]).unwrap();
    /// let entry = BlameEntry::new(hash, 1, 1, "f".into(), None).unwrap();
    /// assert_eq!(entry.commit_id(), hash);
    /// ```
    #[must_use]
    pub const fn commit_id(&self) -> Hash {
        self.commit_id
    }

    /// Returns the first line number (1-based).
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::blame::BlameEntry;
    /// # use libvctrl_handler::Hash;
    /// # let hash = Hash::from_bytes(&[0_u8; 64]).unwrap();
    /// let entry = BlameEntry::new(hash, 42, 1, "f".into(), None).unwrap();
    /// assert_eq!(entry.start_line(), 42);
    /// ```
    #[must_use]
    pub const fn start_line(&self) -> usize {
        self.start_line
    }

    /// Returns the number of lines in this range.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::blame::BlameEntry;
    /// # use libvctrl_handler::Hash;
    /// # let hash = Hash::from_bytes(&[0_u8; 64]).unwrap();
    /// let entry = BlameEntry::new(hash, 1, 5, "f".into(), None).unwrap();
    /// assert_eq!(entry.line_count(), 5);
    /// ```
    #[must_use]
    pub const fn line_count(&self) -> usize {
        self.line_count
    }

    /// Returns the path of the file.
    ///
    /// # How it works
    /// Returns a string slice (`&str`) borrowing from the internal `String`. This avoids
    /// allocation when the caller only needs to read the path.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::blame::BlameEntry;
    /// # use libvctrl_handler::Hash;
    /// # let hash = Hash::from_bytes(&[0_u8; 64]).unwrap();
    /// let entry = BlameEntry::new(hash, 1, 1, "src/main.rs".into(), None).unwrap();
    /// assert_eq!(entry.path(), "src/main.rs");
    /// ```
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Returns an optional summary of the commit message.
    ///
    /// # How it works
    /// Uses `as_deref()` to transparently convert `&Option<String>` to `Option<&str>`,
    /// avoiding the need to clone the `String` if the caller only wishes to read the summary.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::blame::BlameEntry;
    /// # use libvctrl_handler::Hash;
    /// # let hash = Hash::from_bytes(&[0_u8; 64]).unwrap();
    /// let entry = BlameEntry::new(hash, 1, 1, "f".into(), Some("Fix bug".into())).unwrap();
    /// assert_eq!(entry.summary(), Some("Fix bug"));
    /// ```
    #[must_use]
    pub fn summary(&self) -> Option<&str> {
        self.summary.as_deref()
    }
}

/// Trait for computing blame information for files.
///
/// # Why this exists
/// Defines the abstract contract for attributing file lines to commits. By using a trait,
/// the crate decouples the blame algorithm from the repository backend. This allows for
/// different implementations (e.g., a simple linear walker vs. a complex graph traversal
/// that handles merges).
///
/// # Design Rationale: `Send + Sync`
/// The trait requires `Send + Sync` because blame computation is highly parallelizable.
/// File-level blame operations are independent of one another. Implementors can safely
/// distribute `&self` across multiple threads to compute blame for different files
/// concurrently, leveraging multi-core processors without data races.
///
/// # Examples
///
/// Implementing the trait for a mock repository:
///
/// ```
/// # use libvctrl_handler::traits::core::blame::{Blame, BlameEntry};
/// # use libvctrl_handler::{Hash, VctrlError};
/// #
/// struct MockRepo;
///
/// impl Blame for MockRepo {
///     fn blame_file(&self, _path: &str) -> Result<Vec<BlameEntry>, VctrlError> {
///         # let hash = Hash::from_bytes(&[0_u8; 64]).unwrap();
///         let entry = BlameEntry::new(hash, 1, 10, "file.txt".into(), None)?;
///         Ok(vec![entry])
///     }
/// }
///
/// let repo = MockRepo;
/// let entries = repo.blame_file("file.txt").unwrap();
/// assert_eq!(entries.len(), 1);
/// ```
pub trait Blame: Send + Sync {
    /// Returns blame entries for the given file path.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the file cannot be found or the blame calculation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::blame::{Blame, BlameEntry};
    /// # use libvctrl_handler::{Hash, VctrlError};
    /// #
    /// # struct MockRepo;
    /// # impl Blame for MockRepo {
    /// #     fn blame_file(&self, _path: &str) -> Result<Vec<BlameEntry>, VctrlError> {
    /// #         Ok(Vec::new())
    /// #     }
    /// # }
    /// let repo = MockRepo;
    /// assert!(repo.blame_file("nonexistent.txt").is_ok());
    /// ```
    fn blame_file(&self, path: &str) -> Result<Vec<BlameEntry>, VctrlError>;
}
