/// Convenience macro to create a [`VctrlError::Other`] with a formatted message.
///
/// # Example
/// ```rust
/// use libvctrl_handler::vctrl_error_other;
/// let err = vctrl_error_other!("unexpected condition: {}", 42);
/// ```
#[macro_export]
macro_rules! vctrl_error_other {
    ($($arg:tt)*) => {
        $crate::VctrlError::Other(format!($($arg)*))
    };
}
