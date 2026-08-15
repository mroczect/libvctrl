#[macro_export]
macro_rules! vctrl_error_other {
    ($($arg:tt)*) => {
        $crate::VctrlError::Other(format!($($arg)*))
    };
}

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
