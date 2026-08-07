//! Convenience macros for working with errors and other common patterns.

/// Convenience macro to create a [`crate::VctrlError::Other`] with a formatted message.
///
/// # Example
/// ```
/// # use libvctrl_handler::*;
/// let err = vctrl_error_other!("something went wrong: {}", 42);
/// assert_eq!(err.to_string(), "something went wrong: 42");
/// ```
#[macro_export]
macro_rules! vctrl_error_other {
    ($($arg:tt)*) => {
        $crate::VctrlError::Other(format!($($arg)*))
    };
}
