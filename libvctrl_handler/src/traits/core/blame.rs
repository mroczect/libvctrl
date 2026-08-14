//! Defines the `Blame` trait for line-by-line origin tracking.
//!
//! # Purpose
//!
//! The `Blame` trait abstracts the process of annotating each line of a file
//! with the commit that last modified it, similar to `git blame`. This is
//! useful for understanding the history of code and for some porcelain
//! commands that need to display per-line attribution.
//!
//! # Why a separate module
//!
//! Blame computation requires integration with revision walking, tree
//! diffing, and object storage. Keeping the trait in its own file allows
//! different implementations (e.g., simplified or optimized) to be swapped
//! without affecting the rest of the system.
//!
//! # Examples
//!
//! A dummy implementation that returns no entries:
//!
//! ```
//! use libvctrl_handler::{Blame, BlameEntry, Hash, VctrlError};
//!
//! struct DummyBlame;
//!
//! impl Blame for DummyBlame {
//!     type CommitId = Hash;
//!     type Path = String;
//!
//!     fn blame_file(&self, _path: &String) -> Result<Vec<BlameEntry>, VctrlError> {
//!         Ok(vec![])
//!     }
//! }
//!
//! let blame = DummyBlame;
//! let entries = blame.blame_file(&"src/main.rs".to_string()).unwrap();
//! assert!(entries.is_empty());
//! ```

use crate::{Hash, VctrlError};

/// A single line-origin entry produced by the `Blame` trait.
///
/// This struct records which commit last modified a given range of lines,
/// along with the originating path and line numbers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlameEntry {
    /// The commit that last modified these lines.
    pub commit_id: Hash,
    /// The line number where this entry starts in the final file.
    pub start_line: usize,
    /// The number of lines in this entry.
    pub line_count: usize,
    /// The path of the file in the originating commit.
    pub path: String,
    /// A short summary of the commit (e.g., the first line of the message).
    pub summary: Option<String>,
}

/// Trait for line-by-line origin tracking.
///
/// # Purpose
///
/// `Blame` abstracts the ability to determine, for each line of a file,
/// which commit introduced it. This is similar to `git blame` and is
/// useful for code archaeology and history inspection.
///
/// # Associated Types
///
/// - `CommitId`: the type used to identify a commit (e.g., `Hash`).
/// - `Path`: the type used to represent a file path.
///
/// # Examples
///
/// A dummy implementation that returns an empty list:
///
/// ```
/// use libvctrl_handler::{Blame, BlameEntry, Hash, VctrlError};
///
/// struct DummyBlame;
///
/// impl Blame for DummyBlame {
///     type CommitId = Hash;
///     type Path = String;
///
///     fn blame_file(&self, _path: &String) -> Result<Vec<BlameEntry>, VctrlError> {
///         Ok(vec![])
///     }
/// }
///
/// let blame = DummyBlame;
/// assert!(blame.blame_file(&"README.md".to_string()).unwrap().is_empty());
/// ```
///
/// # Errors
///
/// - [`VctrlError::ObjectNotFound`] if the file or its history cannot be
///   found.
/// - [`VctrlError::Other`] if the blame computation fails for any other
///   reason.
pub trait Blame {
    /// The type used to identify a commit.
    type CommitId;

    /// The type used to represent a file path.
    type Path;

    /// Computes line-by-line origin information for the given file.
    ///
    /// # Parameters
    ///
    /// - `path`: the path of the file to blame.
    ///
    /// # Errors
    ///
    /// Returns an error if the file does not exist, its history cannot be
    /// determined, or the underlying storage backend fails.
    fn blame_file(&self, path: &Self::Path) -> Result<Vec<BlameEntry>, VctrlError>;
}
