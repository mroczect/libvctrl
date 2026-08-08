use crate::errors::VctrlError;

use super::validate_name;

/// Identity of a user (author or committer).
///
/// Contains a **name** and an **email**. Both are required to be non‑empty.
/// The name is also validated against [`MAX_NAME_LENGTH`].
///
/// # Example
///
/// ```rust
/// use libvctrl_handler::UserID;
///
/// let user = UserID::new("Alice".into(), "alice@example.com".into())
///     .expect("valid user");
/// assert_eq!(user.name(), "Alice");
/// assert_eq!(user.email(), "alice@example.com");
///
/// // Empty fields are rejected.
/// assert!(UserID::new("".into(), "x@y".into()).is_err());
/// assert!(UserID::new("Alice".into(), "".into()).is_err());
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserID {
    name: String,
    email: String,
}

impl UserID {
    /// Creates a new `UserID` after validating name and email.
    ///
    /// # Errors
    /// Returns [`VctrlError::InvalidName`] if:
    /// - `name` is empty or too long.
    /// - `email` is empty.
    pub fn new(name: String, email: String) -> Result<Self, VctrlError> {
        validate_name(&name)?;
        if email.is_empty() {
            return Err(VctrlError::InvalidName("email is empty".into()));
        }
        Ok(Self { name, email })
    }

    /// Returns the user's name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the user's email.
    #[must_use]
    pub fn email(&self) -> &str {
        &self.email
    }
}
