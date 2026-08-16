/// Constructs a [`VctrlError::Other`](crate::VctrlError::Other) from a format string and arguments.
///
/// # Why this exists
/// In Rust, formatting a string and wrapping it into a custom error variant often requires
/// verbose syntax like `VctrlError::Other(format!(...))`. This declarative macro provides
/// syntactic sugar to eliminate this boilerplate. It ensures that ad-hoc errors are
/// constructed consistently and concisely across the codebase, mirroring the ergonomics
/// of the standard library's `println!` or `format!` macros.
///
/// # How it works
/// Under the hood, this macro delegates to the standard `format!` macro to allocate
/// a new `String` on the heap. It then wraps this `String` in the
/// [`VctrlError::Other`](crate::VctrlError::Other) variant.
///
/// The use of `$crate` in the expansion is critical. It guarantees that the path to
/// `VctrlError` resolves correctly to this crate's root, even if the macro is invoked
/// from an external crate that has brought the macro into scope via a glob import.
/// This prevents shadowing issues and ensures absolute path resolution without requiring
/// the consumer to manually import the error enum alongside the macro.
///
/// # Examples
///
/// Creating a simple error message:
///
/// ```
/// # use my_crate::{VctrlError, vctrl_error_other};
/// let err = vctrl_error_other!("file not found");
/// assert_eq!(err.to_string(), "file not found");
/// ```
///
/// Formatting arguments into the error message:
///
/// ```
/// # use my_crate::{VctrlError, vctrl_error_other};
/// let filename = "config.toml";
/// let code = 404;
/// let err = vctrl_error_other!("missing configuration file: {} (code {})", filename, code);
/// assert_eq!(err.to_string(), "missing configuration file: config.toml (code 404)");
/// ```
#[macro_export]
macro_rules! vctrl_error_other {
    ($($arg:tt)*) => {
        $crate::VctrlError::Other(format!($($arg)*))
    };
}
