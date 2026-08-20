#![allow(unreachable_pub)]
#![allow(dead_code)]
#![allow(clippy::panic)]

pub fn ok<T, E: core::fmt::Debug>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(err) => panic!("expected Ok(..), got Err({err:?})"),
    }
}

pub fn err<T: core::fmt::Debug, E: core::fmt::Debug>(result: Result<T, E>) -> E {
    match result {
        Ok(value) => panic!("expected Err(..), got Ok({value:?})"),
        Err(err) => err,
    }
}
