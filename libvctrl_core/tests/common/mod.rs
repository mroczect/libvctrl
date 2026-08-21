use libvctrl_handler::{Hash, VctrlError};

pub const fn make_hash(byte: u8) -> Result<Hash, VctrlError> {
    Hash::from_bytes(&[byte; 64])
}
