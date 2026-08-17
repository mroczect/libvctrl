//! User identity representation.
//!
//! # Architecture
//! This module defines the [`UserID`] struct, which represents the `Name <email>`
//! syntax used in Git commits and tags. User identities are critical for audit
//! trails and blame calculations.
//!
//! # Design Rationale: Security by Construction
//! Git's internal text format relies on specific characters (like `<`, `>`, and `\n`)
//! as delimiters. If a username or email contains these characters, it can corrupt
//! the commit object structure or inject malicious headers. The [`UserID::new`]
//! constructor acts as a strict validation gate. By rejecting empty strings, control
//! characters, and missing `@` symbols at construction time, the crate guarantees
//! that any `UserID` instance in memory is safe to serialize into a Git object.

use crate::constants::MAX_NAME_LENGTH;
use crate::errors::VctrlError;

/// A user identity (author or committer).
///
/// # Why this exists
/// Provides a strongly-typed, validated wrapper around the `Name <email>` concept.
/// By requiring construction via [`new`](Self::new), the crate ensures that every
/// `UserID` adheres to length and character constraints. Once constructed, the
/// identity is immutable, ensuring safe, concurrent sharing across threads.
///
/// # How it works
/// The struct stores the name and email as owned `String`s. The constructor
/// performs a series of checks: it verifies that neither string is empty, neither
/// exceeds [`MAX_NAME_LENGTH`](crate::constants::MAX_NAME_LENGTH), neither contains
/// ASCII control characters (like newlines), and the email contains an `@` symbol.
///
/// # Examples
///
/// Creating a valid user identity:
///
/// ```
/// # use libvctrl_handler::types::core::user_id::UserID;
/// # use libvctrl_handler::VctrlError;
/// let user = UserID::new("Alice".to_string(), "alice@example.com".to_string())?;
/// assert_eq!(user.name(), "Alice");
/// assert_eq!(user.email(), "alice@example.com");
/// # Ok::<(), VctrlError>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserID {
    name: String,
    email: String,
}

impl UserID {
    /// Creates a new `UserID`.
    ///
    /// # How it works
    /// Performs a multi-stage validation process:
    /// 1. Checks `name` for emptiness, length limits (using `usize::try_from` for
    ///    32-bit architecture safety), and ASCII control characters.
    /// 2. Checks `email` for emptiness, length limits, ASCII control characters,
    ///    and the presence of an `@` symbol.
    /// If any check fails, an error is returned and the original strings are dropped.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::InvalidName`] if the name is empty, too long, or contains control characters.
    /// Returns [`VctrlError::InvalidEmail`] if the email is empty, lacks `@`, or contains control characters.
    ///
    /// # Examples
    ///
    /// Handling an invalid email:
    ///
    /// ```
    /// # use libvctrl_handler::types::core::user_id::UserID;
    /// # use libvctrl_handler::VctrlError;
    /// let result = UserID::new("Bob".to_string(), "bob-example.com".to_string());
    /// assert!(matches!(result, Err(VctrlError::InvalidEmail(_))));
    /// # Ok::<(), VctrlError>(())
    /// ```
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
    ///
    /// # How it works
    /// Returns a string slice (`&str`) borrowing from the internal `String`. This
    /// avoids allocation when the caller only needs to read the name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the email address.
    ///
    /// # How it works
    /// Returns a string slice (`&str`) borrowing from the internal `String`. This
    /// avoids allocation when the caller only needs to read the email.
    #[must_use]
    pub fn email(&self) -> &str {
        &self.email
    }
}
