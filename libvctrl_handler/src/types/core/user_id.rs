//! User identity type.
//!
//! [`UserID`] captures the name and email of an actor (author or committer)
//! in the version control system. It enforces basic validity rules at
//! construction to guarantee that stored identities are always well‑formed.

use crate::errors::VctrlError;
use crate::types::validate_name;

/// A validated user identity consisting of a name and an email address.
///
/// `UserID` is a simple value object that carries two pieces of information
/// required to record who performed an action. The fields are private to
/// ensure that no invalid state can be introduced after construction.
///
/// # Why validation at construction?
///
/// Constructors that return `Result` force callers to handle malformed
/// input immediately. Once a `UserID` exists, the system can rely on its
/// invariants without repeated checks elsewhere.
///
/// # Invariants
///
/// - `name` is non‑empty and does not exceed the system’s maximum name
///   length (enforced by [`validate_name`]).
/// - `email` is non‑empty.
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
    /// * `name` - The display name of the user. Must be non‑empty and not
    ///   exceed the maximum name length.
    /// * `email` - The email address of the user. Must be non‑empty. This
    ///   implementation does not perform full RFC‑822 validation, only a
    ///   basic emptiness check.
    ///
    /// # Errors
    ///
    /// - [`VctrlError::InvalidName`] if `name` is empty or too long.
    /// - [`VctrlError::InvalidEmail`] if `email` is empty.
    ///
    /// # Examples
    ///
    /// Successful creation:
    ///
    /// ```
    /// use libvctrl_handler::types::core::UserID;
    ///
    /// let user = UserID::new("Bob".into(), "bob@example.com".into()).unwrap();
    /// ```
    ///
    /// Empty name:
    ///
    /// ```
    /// use libvctrl_handler::types::core::UserID;
    ///
    /// let err = UserID::new("".into(), "bob@example.com".into()).unwrap_err();
    /// assert!(matches!(err, libvctrl_handler::errors::VctrlError::InvalidName(_)));
    /// ```
    ///
    /// Empty email:
    ///
    /// ```
    /// use libvctrl_handler::types::core::UserID;
    ///
    /// let err = UserID::new("Bob".into(), "".into()).unwrap_err();
    /// assert!(matches!(err, libvctrl_handler::errors::VctrlError::InvalidEmail(_)));
    /// ```
    pub fn new(name: String, email: String) -> Result<Self, VctrlError> {
        validate_name(&name)?;
        if email.is_empty() {
            return Err(VctrlError::InvalidEmail("email is empty".into()));
        }
        Ok(Self { name, email })
    }

    /// Returns the user's display name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the user's email address.
    #[must_use]
    pub fn email(&self) -> &str {
        &self.email
    }
}
