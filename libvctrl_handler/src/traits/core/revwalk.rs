//! Revision walking trait.
//!
//! # Architecture
//! This module provides the contract for traversing the commit graph. Walking
//! history is a fundamental operation for log generation, bisecting, and ancestry
//! queries. By abstracting this into a trait, the crate allows backends to implement
//! optimized traversal algorithms (e.g., topological sorting, priority queues based
//! on timestamps) without leaking those implementation details to the caller.
//!
//! # Design Rationale: Lazy Evaluation
//! Repositories like the Linux kernel contain millions of commits. Loading the
//! entire commit graph into memory at once would cause severe memory exhaustion.
//! The [`RevWalk::walk`] method returns an iterator, enforcing lazy evaluation.
//! Commits are only loaded and yielded from the underlying object store as the
//! iterator is consumed, maintaining a constant, predictable memory footprint.

use crate::errors::VctrlError;

/// An iterator over commit history.
///
/// # Why this exists
/// This type alias standardizes the return type of revision walks across all
/// backends. It uses dynamic dispatch (`Box<dyn Iterator>`) to perform type erasure.
/// This allows a backend to return any complex internal iterator struct (e.g., a
/// binary heap for priority-ordered traversal) without forcing the caller to know
/// the concrete type or bloating the trait signature with associated types.
///
/// # How it works
/// - `Item = Result<T, VctrlError>`: Yields a `Result` because graph traversal may
///   encounter I/O errors (e.g., a missing commit object) mid-iteration.
/// - `Send`: The iterator can be safely transferred across threads, enabling
///   parallel processing of commit history (e.g., using `rayon`).
/// - `'a`: The lifetime ties the iterator to the lifetime of the [`RevWalk`]
///   instance that created it, ensuring the backend store remains valid while
///   the iterator is active.
pub type RevWalkIterator<'a, T> = Box<dyn Iterator<Item = Result<T, VctrlError>> + Send + 'a>;

/// Trait for walking commit history.
///
/// # Why this exists
/// Provides a unified interface for commit graph traversal. By using an associated
/// type for the commit identifier, the trait is not hardcoded to cryptographic
/// hashes. An in-memory testing backend might use array indices (`usize`), while
/// a disk-backed backend uses [`Hash`](crate::Hash).
///
/// # How it works
/// The `walk` method accepts a starting commit identifier and returns a
/// [`RevWalkIterator`]. The implementor is responsible for resolving the start
/// commit, reading its parent hashes, and pushing them into an internal queue.
/// As the caller calls `next()` on the iterator, the backend dequeues a commit,
/// fetches its parents, and yields the commit.
///
/// # Design Rationale: `&self` on `walk`
/// Note that `walk` takes `&self` instead of `&mut self`. Traversal is a read-only
/// operation from the perspective of the walker's state. The implementor must use
/// interior mutability (e.g., `Mutex` for internal buffers) if the underlying
/// object store requires mutable access to read objects, allowing multiple
/// concurrent walks to occur safely.
///
/// # Examples
///
/// Implementing the trait for a mock graph:
///
/// ```
/// # use libvctrl_handler::traits::core::revwalk::{RevWalk, RevWalkIterator};
/// # use libvctrl_handler::VctrlError;
/// #
/// struct MockRevWalk;
///
/// impl RevWalk for MockRevWalk {
///     type CommitId = u32;
///
///     fn walk(&self, start: &Self::CommitId) -> Result<RevWalkIterator<'_, Self::CommitId>, VctrlError> {
///         let start = *start;
///         // Simulate walking backwards through commit IDs 0 to `start`
///         Ok(Box::new((0..start).rev().map(Ok)))
///     }
/// }
///
/// let walker = MockRevWalk;
/// let iter = walker.walk(&3)?;
/// let commits: Vec<u32> = iter.filter_map(|c| c.ok()).collect();
/// assert_eq!(commits, vec![2, 1, 0]);
/// # Ok::<(), VctrlError>(())
/// ```
pub trait RevWalk: Send + Sync {
    /// The commit identifier type.
    ///
    /// # Why this exists
    /// Decouples the traversal logic from the identifier format. While typically
    /// a 64-byte [`Hash`](crate::Hash), this allows specialized backends to use
    /// more efficient representations like integers or pointers.
    type CommitId: Send + Sync;

    /// Returns an iterator over commit history starting from the given commit.
    ///
    /// # How it works
    /// Resolves the `start` commit and initializes an iterator. The iterator
    /// traverses the graph (typically in reverse chronological order, respecting
    /// topological constraints). The lifetime `'_` binds the returned iterator to
    /// the `RevWalk` implementor, ensuring the backend is not dropped prematurely.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the starting commit cannot be found in the
    /// underlying store, or if initializing the traversal queue fails.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::revwalk::{RevWalk, RevWalkIterator};
    /// # use libvctrl_handler::VctrlError;
    /// # struct MockRevWalk;
    /// # impl RevWalk for MockRevWalk {
    /// #     type CommitId = u32;
    /// #     fn walk(&self, s: &Self::CommitId) -> Result<RevWalkIterator<'_, Self::CommitId>, VctrlError> {
    /// #         Ok(Box::new((0..*s).rev().map(Ok)))
    /// #     }
    /// # }
    /// let walker = MockRevWalk;
    /// let mut iter = walker.walk(&5)?;
    /// assert_eq!(iter.next(), Some(Ok(4)));
    /// # Ok::<(), VctrlError>(())
    /// ```
    fn walk(
        &self,
        start: &Self::CommitId,
    ) -> Result<RevWalkIterator<'_, Self::CommitId>, VctrlError>;
}
