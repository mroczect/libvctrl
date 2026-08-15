//! Helper macros for the crate.

/// Constructs a [`VctrlError::Other`](crate::VctrlError::Other) from a format string and arguments.
///
/// # Examples
/// ```
/// let err = vctrl_error_other!("failed to parse {}", input);
/// ```
#[macro_export]
macro_rules! vctrl_error_other {
    ($($arg:tt)*) => {
        $crate::VctrlError::Other(format!($($arg)*))
    };
}
