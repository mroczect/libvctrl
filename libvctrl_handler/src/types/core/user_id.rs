//! User identifier type.

use crate::errors::VctrlError;
use crate::types::validate_name;

/// A Git author/committer identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserID {
    name: String,
    email: String,
}

impl UserID {
    /// Creates a new user identity.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::InvalidName`] if the name is invalid,
    /// or [`VctrlError::InvalidEmail`] if the email is empty.
    pub fn new(name: String, email: String) -> Result<Self, VctrlError> {
        validate_name(&name)?;
        if email.is_empty() {
            return Err(VctrlError::InvalidEmail("email is empty".into()));
        }
        Ok(Self { name, email })
    }

    /// Returns the user name.
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
