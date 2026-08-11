//! User identity type for version control systems.
//!
//! A [`UserID`] pairs a human‑readable name with an email address, both of
//! which are validated at construction time to ensure they conform to the
//! repository's constraints.

use crate::errors::VctrlError;

use super::validate_name;

/// Identifies a person in the version control history.
///
/// `UserID` is used in commits and tags to record who authored or applied a
/// change. It consists of a display `name` (non‑empty, within the maximum
/// length) and an `email` address (non‑empty). Once constructed, a `UserID`
/// is immutable – all fields are private and only accessible via read‑only
/// accessors.
///
/// # Design
///
/// Both the name and email are validated by the constructor:
/// - The name is checked by the internal `validate_name` function, which
///   ensures it is non‑empty and does not exceed the maximum allowed length
///   (see [`MAX_NAME_LENGTH`](crate::constants::MAX_NAME_LENGTH)).
/// - The email must not be empty. Additional format validation may be added
///   in the future without breaking the public API because the fields remain
///   private.
///
/// This strict construction guarantees that every `UserID` in the system is
/// valid, simplifying downstream code that consumes these identities.
///
/// # Examples
///
/// Creating a valid user identity:
///
/// ```
/// # use libvctrl_handler::UserID;
/// let user = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();
/// assert_eq!(user.name(), "Alice");
/// assert_eq!(user.email(), "alice@example.com");
/// ```
///
/// An empty name or email causes an error:
///
/// ```
/// # use libvctrl_handler::UserID;
/// assert!(UserID::new("".into(), "a@b.com".into()).is_err());
/// assert!(UserID::new("Alice".into(), "".into()).is_err());
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserID {
    name: String,
    email: String,
}

impl UserID {
    /// Creates a new [`UserID`] after validating the name and email.
    ///
    /// The `name` must be non‑empty and not exceed the maximum name length
    /// defined by [`MAX_NAME_LENGTH`](crate::constants::MAX_NAME_LENGTH).
    /// The `email` must be non‑empty. No other format checks are performed.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::InvalidName`] if:
    /// - `name` is empty or too long, or
    /// - `email` is empty.
    ///
    /// # Examples
    ///
    /// Successful construction:
    ///
    /// ```
    /// # use libvctrl_handler::UserID;
    /// let user = UserID::new("Bob".into(), "bob@example.com".into()).unwrap();
    /// assert_eq!(user.name(), "Bob");
    /// ```
    pub fn new(name: String, email: String) -> Result<Self, VctrlError> {
        validate_name(&name)?;
        if email.is_empty() {
            return Err(VctrlError::InvalidEmail("email is empty".into())); // CHANGED
        }
        Ok(Self { name, email })
    }

    /// Returns the display name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the email address.
    #[must_use]
    pub fn email(&self) -> &str {
        &self.email
    }
}
