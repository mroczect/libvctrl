//! Convenience macros for `libvctrl_handler`.
//!
//! # Purpose
//!
//! This module provides macros that simplify common error-handling patterns
//! used throughout the crate. They are exported with `#[macro_export]` and
//! can therefore be used by downstream code as well as by internal modules.
//!
//! # Design Rationale
//!
//! - **`vctrl_error_other!`** reduces boilerplate when constructing the
//!   catch-all [`VctrlError::Other`](crate::VctrlError::Other) variant. It
//!   mirrors the standard [`format!`] syntax, making call sites familiar.
//! - **`string_payload_variants!`** centralizes a repetitive `match` pattern
//!   used by the [`PartialEq`] implementation of [`VctrlError`]. Instead of
//!   manually extracting string payloads from many variants, the macro
//!   generates a private helper function.
//!
//! # Macro Hygiene
//!
//! Both macros use `$crate`-qualified paths where appropriate. This ensures
//! that the generated code refers to the correct crate even when the macros
//! are re-exported or used from downstream crates with different names.
//!
//! # Internal Mechanism
//!
//! [`vctrl_error_other!`] performs a standard token expansion: it wraps the
//! result of `format!` directly in [`VctrlError::Other`].
//!
//! [`string_payload_variants!`] accepts a comma-separated list of variant
//! identifiers. It expands into a `const fn string_payload` that matches each
//! listed variant and returns `Some(s.as_str())`. The generated function is
//! scoped to the location where the macro is invoked, typically inside the
//! `eq` method of `impl PartialEq for VctrlError`.
//!
//! # Examples
//!
//! Constructing a formatted error:
//!
//! ```
//! use libvctrl_handler::vctrl_error_other;
//!
//! let err = vctrl_error_other!("failed to open '{}': {}", "config.toml", "permission denied");
//! assert_eq!(
//!     err.to_string(),
//!     "failed to open 'config.toml': permission denied"
//! );
//! ```

/// Creates a [`VctrlError::Other`] variant with a formatted message.
///
/// This macro is a shorthand for building miscellaneous errors without
/// manually calling `format!`. It accepts the same arguments as `format!`
/// and wraps the result in [`VctrlError::Other`].
///
/// # Design Rationale
///
/// Error construction is frequent in fallible code. By providing a macro,
/// callers can avoid the visual noise of `VctrlError::Other(format!(...))`
/// and instead write a single concise invocation.
///
/// # How It Works
///
/// The macro expands to:
///
/// ```text
/// $crate::VctrlError::Other(format!($($arg)*))
/// ```
///
/// The use of `$crate` guarantees that the macro resolves the correct
/// `VctrlError` type even if the macro is called from a downstream crate
/// that imports the macro under a different name.
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
///
/// The returned value is a [`VctrlError`](crate::VctrlError), so it can be
/// propagated with the `?` operator in functions returning
/// [`Result<T, VctrlError>`](crate::VctrlError):
///
/// ```
/// # use libvctrl_handler::{vctrl_error_other, VctrlError};
/// fn fallible(code: u32) -> Result<(), VctrlError> {
///     if code != 0 {
///         return Err(vctrl_error_other!("non-zero exit code: {code}"));
///     }
///     Ok(())
/// }
///
/// assert!(fallible(1).is_err());
/// assert!(fallible(0).is_ok());
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
/// [`VctrlError`](crate::VctrlError) to extract the string payload from all
/// variants that carry a [`String`]. It must be exported because it is
/// invoked from the `errors` module.
///
/// # Design Rationale
///
/// [`VctrlError`](crate::VctrlError) has several string-bearing variants:
/// [`InvalidName`](crate::VctrlError::InvalidName),
/// [`InvalidEmail`](crate::VctrlError::InvalidEmail),
/// [`RefNotFound`](crate::VctrlError::RefNotFound),
/// [`CorruptedData`](crate::VctrlError::CorruptedData),
/// [`SerializationError`](crate::VctrlError::SerializationError), and
/// [`Other`](crate::VctrlError::Other). In the `eq` method, these variants
/// must be compared by their string content. The macro avoids repeating the
/// same `match` arm for every variant.
///
/// # How It Works
///
/// The macro accepts a list of variant identifiers and expands to a local
/// `const fn string_payload(v: &VctrlError) -> Option<&str>` that returns
/// `Some(s.as_str())` for the listed variants and `None` otherwise. The
/// function is generated at the invocation site, so the surrounding scope
/// must already have `VctrlError` in scope.
///
/// # Examples
///
/// The macro can be used with a locally defined error enum to generate a
/// payload extractor:
///
/// ```
/// use libvctrl_handler::string_payload_variants;
///
/// enum VctrlError {
///     InvalidName(String),
///     RefNotFound(String),
/// }
///
/// string_payload_variants!(InvalidName, RefNotFound);
///
/// let err = VctrlError::InvalidName("bad".to_string());
/// assert_eq!(string_payload(&err), Some("bad"));
///
/// let other = VctrlError::RefNotFound("main".to_string());
/// assert_eq!(string_payload(&other), Some("main"));
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
