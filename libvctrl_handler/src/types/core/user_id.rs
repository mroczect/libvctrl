//! User identity type.
//!
//! # Purpose
//!
//! `UserID` captures the name and email of an actor (author or committer)
//! in the version control system. It enforces basic validity rules at
//! construction to guarantee that stored identities are always well-formed.
//!
//! # Design Rationale
//!
//! Identity is essential for recording who authored or committed a change.
//! A `UserID` is a small value object that bundles a display name and an
//! email address. By validating these fields at construction, the rest of
//! the system can rely on their invariants without repeated checks.
//!
//! ## Why not just use `(String, String)`?
//!
//! A dedicated type provides:
//!
//! - **Semantic clarity**: Callers know that a value represents a user
//!   identity, not an arbitrary pair of strings.
//! - **Encapsulation**: The fields are private, so they cannot be mutated
//!   after construction.
//! - **Validation**: The constructor enforces invariants, preventing invalid
//!   data from entering the system.
//!
//! # Relationship to Other Types
//!
//! `UserID` is used by `Commit` and `Tag`
//! to record author, committer, and tagger identities.
//!
//! # Memory Layout
//!
//! A `UserID` owns two heap-allocated `String`s. Its size is exactly
//! two `String` sizes (48 bytes on 64-bit platforms). Cloning performs a
//! deep copy of both strings.
//!
//! # Examples
//!
//! ```
//! use libvctrl_handler::types::core::UserID;
//!
//! let user = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();
//! assert_eq!(user.name(), "Alice");
//! assert_eq!(user.email(), "alice@example.com");
//! ```

use crate::errors::VctrlError;
use crate::types::validate_name;

/// A validated user identity consisting of a name and an email address.
///
/// # Purpose
///
/// `UserID` is a simple value object that carries two pieces of information
/// required to record who performed an action: the display name and the
/// email address. The fields are private to ensure that no invalid state
/// can be introduced after construction.
///
/// # Design Rationale
///
/// - **Validation at construction**: The constructor returns a `Result`,
///   forcing callers to handle malformed input immediately. Once a `UserID`
///   exists, the system can rely on its invariants without repeated checks
///   elsewhere.
/// - **Immutability**: The fields are private and no mutable accessors are
///   provided, preserving the identity's integrity.
/// - **Cloneable and comparable**: The struct derives `Clone`, `Debug`,
///   `PartialEq`, and `Eq`, making it easy to duplicate, print, and
///   compare identities.
///
/// # Invariants
///
/// - `name` is non-empty and does not exceed the system's maximum name
///   length (enforced by `validate_name`).
/// - `email` is non-empty. Full RFC-compliant email validation is not
///   performed; only a basic emptiness check is enforced.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::types::core::UserID;
///
/// let user = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();
/// assert_eq!(user.name(), "Alice");
/// assert_eq!(user.email(), "alice@example.com");
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserID {
    name: String,
    email: String,
}

impl UserID {
    /// Creates a new `UserID` after validating the name and email.
    ///
    /// # Arguments
    ///
    /// * `name` - The display name of the user. Must be non-empty and not
    ///   exceed the maximum name length.
    /// * `email` - The email address of the user. Must be non-empty. This
    ///   implementation does not perform full RFC-822 validation, only a
    ///   basic emptiness check.
    ///
    /// # Errors
    ///
    /// - `VctrlError::InvalidName` if `name` is empty or too long.
    /// - `VctrlError::InvalidEmail` if `email` is empty.
    ///
    /// # Why only basic email validation?
    ///
    /// Full email validation is complex and varies by standard. The crate
    /// intentionally enforces only a minimal invariant: the email must not be
    /// empty. This prevents accidental omissions while remaining permissive
    /// enough for future extension or integration with more strict validators.
    ///
    /// # How It Works Internally
    ///
    /// 1. Calls `validate_name` on the provided name. If invalid, returns
    ///    an error.
    /// 2. Checks whether the email is empty. If so, returns
    ///    `VctrlError::InvalidEmail`.
    /// 3. Constructs and returns `Ok(Self { name, email })`.
    ///
    /// # Examples
    ///
    /// Successful creation:
    ///
    /// ```
    /// use libvctrl_handler::types::core::UserID;
    ///
    /// let user = UserID::new("Bob".into(), "bob@example.com".into()).unwrap();
    /// assert_eq!(user.name(), "Bob");
    /// assert_eq!(user.email(), "bob@example.com");
    /// ```
    ///
    /// Empty name:
    ///
    /// ```
    /// use libvctrl_handler::types::core::UserID;
    ///
    /// let err = UserID::new("".into(), "bob@example.com".into()).unwrap_err();
    /// assert!(matches!(err, libvctrl_handler::VctrlError::InvalidName(_)));
    /// ```
    ///
    /// Empty email:
    ///
    /// ```
    /// use libvctrl_handler::types::core::UserID;
    ///
    /// let err = UserID::new("Bob".into(), "".into()).unwrap_err();
    /// assert!(matches!(err, libvctrl_handler::VctrlError::InvalidEmail(_)));
    /// ```
    pub fn new(name: String, email: String) -> Result<Self, VctrlError> {
        validate_name(&name)?;
        if email.is_empty() {
            return Err(VctrlError::InvalidEmail("email is empty".into()));
        }
        Ok(Self { name, email })
    }

    /// Returns the user's display name.
    ///
    /// # Returns
    ///
    /// A string slice containing the validated display name.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::types::core::UserID;
    ///
    /// let user = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();
    /// assert_eq!(user.name(), "Alice");
    /// ```
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the user's email address.
    ///
    /// # Returns
    ///
    /// A string slice containing the non-empty email address.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::types::core::UserID;
    ///
    /// let user = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();
    /// assert_eq!(user.email(), "alice@example.com");
    /// ```
    #[must_use]
    pub fn email(&self) -> &str {
        &self.email
    }
}
