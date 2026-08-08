//! Convenience macros for working with errors and other common patterns.
//!
//! This module provides ergonomic macros that reduce boilerplate when using the
//! [`VctrlError`](crate::VctrlError) type and other core facilities.  Currently
//! the only exported macro is [`vctrl_error_other!`], which creates an
//! [`Other`](crate::VctrlError::Other) variant with a formatted message.
//!
//! Additional macros (e.g., for validation or for constructing specific error
//! variants) may be added here in the future without polluting the root crate
//! namespace.

/// Convenience macro to create a [`VctrlError::Other`](crate::VctrlError::Other)
/// with a formatted message.
///
/// This macro accepts a **format string literal** followed by zero or more
/// arguments, exactly like [`format!`].  It does **not** accept an already‑computed
/// `String`; use `VctrlError::Other(…)` directly for that case.
///
/// # When to use `Other` vs a specialised variant
///
/// The `Other` variant is a **fallback** – it should be used only when no
/// existing variant describes the error precisely.  In order of preference:
///
/// 1. Use the most specific variant (`InvalidName`, `CorruptedData`, …)
///    if one matches the situation.
/// 2. If no variant fits, use `vctrl_error_other!` to create a descriptive
///    error message.  This is common in application‑layer code where the
///    standard vocabulary of `VctrlError` does not cover the specific failure.
///
/// # Examples
///
/// Basic usage:
/// ```rust
/// use libvctrl_handler::vctrl_error_other;
///
/// let err = vctrl_error_other!("file '{}' is too large: {} bytes", "data.bin", 12345);
/// assert_eq!(err.to_string(), "file 'data.bin' is too large: 12345 bytes");
/// ```
///
/// Returning from a function:
/// ```rust
/// # use libvctrl_handler::{vctrl_error_other, VctrlError};
/// fn do_something() -> Result<(), VctrlError> {
///     let path = "config.toml";
///     if path.ends_with(".toml") {
///         return Err(vctrl_error_other!("unsupported config format: {path}"));
///     }
///     Ok(())
/// }
/// ```
///
/// Using an already‑existing `String` (no format arguments):
/// ```rust
/// # use libvctrl_handler::VctrlError;
/// let msg = String::from("something bad happened");
/// let err = VctrlError::Other(msg);  // directly, no macro
/// assert_eq!(err.to_string(), "something bad happened");
/// ```
///
/// # Implementation note
///
/// The macro expands to `$crate::VctrlError::Other(format!($($arg)*))`.  It is
/// `#[macro_export]` so it can be used from other crates that depend on
/// `libvctrl_handler` without an explicit `use` of the macro itself
/// (though a `use libvctrl_handler::vctrl_error_other;` is recommended for
/// clarity).
#[macro_export]
macro_rules! vctrl_error_other {
    ($($arg:tt)*) => {
        $crate::VctrlError::Other(format!($($arg)*))
    };
}
