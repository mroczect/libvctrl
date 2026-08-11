//! Convenience macros for `libvctrl_handler`.
//!
//! This module provides macros that simplify common error‑handling patterns
//! used throughout the crate. They are exported and can be used by downstream
//! code.

/// Creates a [`VctrlError::Other`] variant with a formatted message.
///
/// This macro is a shorthand for building miscellaneous errors without
/// manually calling `format!`. It accepts the same arguments as `format!`
/// and wraps the result in [`VctrlError::Other`].
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// # use libvctrl_handler::vctrl_error_other;
/// let err = vctrl_error_other!("failed to open '{}': {}", "config.toml", "permission denied");
/// assert_eq!(
///     err.to_string(),
///     "failed to open 'config.toml': permission denied"
/// );
/// ```
///
/// Using with format specifiers:
///
/// ```
/// # use libvctrl_handler::vctrl_error_other;
/// let code = 42;
/// let err = vctrl_error_other!("unexpected exit code {code}");
/// assert_eq!(err.to_string(), "unexpected exit code 42");
/// ```
#[macro_export]
macro_rules! vctrl_error_other {
    ($($arg:tt)*) => {
        $crate::VctrlError::Other(format!($($arg)*))
    };
}

/// Helper macro to generate the `string_payload` function for [`VctrlError`].
///
/// This macro is used inside the [`PartialEq`] implementation of
/// [`VctrlError`] to extract the string payload from all variants that carry
/// a [`String`]. It must be exported because it is invoked from the
/// `errors` module.
///
/// # Example of invocation
///
/// ```ignore
/// string_payload_variants!(InvalidName, RefNotFound, InvalidEmail, ...);
/// ```
#[macro_export]
macro_rules! string_payload_variants {
    ($($variant:ident),* $(,)?) => {
        const fn string_payload(v: &VctrlError) -> Option<&str> {
            match v {
                $( VctrlError::$variant(s) => Some(s.as_str()), )*
                _ => None,
            }
        }
    };
}
