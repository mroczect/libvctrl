//! Helper macros for the crate.

/// Constructs a [`VctrlError::Other`](crate::VctrlError::Other) from a format string and arguments.
#[macro_export]
macro_rules! vctrl_error_other {
    ($($arg:tt)*) => {
        $crate::VctrlError::Other(format!($($arg)*))
    };
}
