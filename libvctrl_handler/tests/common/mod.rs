#[allow(dead_code, clippy::panic)]
pub(crate) fn ok<T, E: core::fmt::Debug>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(err) => panic!("expected Ok(..), got Err({err:?})"),
    }
}

#[allow(dead_code, clippy::panic)]
pub(crate) fn err<T: core::fmt::Debug, E: core::fmt::Debug>(result: Result<T, E>) -> E {
    match result {
        Ok(value) => panic!("expected Err(..), got Ok({value:?})"),
        Err(err) => err,
    }
}
