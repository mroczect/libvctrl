use crate::constants::MAX_NAME_LENGTH;
use crate::errors::VctrlError;

/// A user identity (author or committer).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserID {
    name: String,
    email: String,
}

impl UserID {
    /// Creates a new `UserID`.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::InvalidName`] if the name is empty, too long, or contains control characters.
    /// Returns [`VctrlError::InvalidEmail`] if the email is empty, lacks `@`, or contains control characters.
    pub fn new(name: String, email: String) -> Result<Self, VctrlError> {
        let max_len = usize::try_from(MAX_NAME_LENGTH).unwrap_or(usize::MAX);
        if name.is_empty() {
            return Err(VctrlError::InvalidName("user name is empty".into()));
        }
        if name.len() > max_len {
            return Err(VctrlError::InvalidName(format!(
                "user name exceeds maximum length {MAX_NAME_LENGTH}"
            )));
        }
        if name.bytes().any(|b| b.is_ascii_control()) {
            return Err(VctrlError::InvalidName(format!(
                "user name contains control characters: '{name}'"
            )));
        }
        if email.is_empty() {
            return Err(VctrlError::InvalidEmail("email is empty".into()));
        }
        if email.len() > max_len {
            return Err(VctrlError::InvalidEmail(format!(
                "email exceeds maximum length {MAX_NAME_LENGTH}"
            )));
        }
        if !email.contains('@') {
            return Err(VctrlError::InvalidEmail(format!(
                "email must contain '@': '{email}'"
            )));
        }
        if email.bytes().any(|b| b.is_ascii_control()) {
            return Err(VctrlError::InvalidEmail(format!(
                "email contains control characters: '{email}'"
            )));
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
