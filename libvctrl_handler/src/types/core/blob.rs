use crate::constants::MAX_BLOB_SIZE;
use crate::errors::VctrlError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Blob {
    data: Vec<u8>,
}

impl Blob {
    pub fn new(data: Vec<u8>) -> Result<Self, VctrlError> {
        let max_size = usize::try_from(MAX_BLOB_SIZE).unwrap_or(usize::MAX);
        if data.len() > max_size {
            return Err(VctrlError::ExceededMaxSize(format!(
                "blob size {} exceeds maximum allowed size {}",
                data.len(),
                MAX_BLOB_SIZE
            )));
        }
        Ok(Self { data })
    }

    #[must_use]
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    #[must_use]
    pub const fn size(&self) -> usize {
        self.data.len()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}
