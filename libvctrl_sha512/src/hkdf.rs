use crate::hmac::HMAC;

impl_hkdf!(crate::sha512::Hash, 64, 128);
