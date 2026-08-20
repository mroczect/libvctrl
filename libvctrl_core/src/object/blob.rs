use libvctrl_handler::{Blob, VctrlError};

#[derive(Debug, Default)]
pub struct BlobBuilder {
    data: Vec<u8>,
}

impl BlobBuilder {
    #[must_use]
    pub const fn new() -> Self {
        Self { data: Vec::new() }
    }

    #[must_use]
    pub fn with_data(mut self, data: Vec<u8>) -> Self {
        self.data = data;
        self
    }

    pub fn build(self) -> Result<Blob, VctrlError> {
        Blob::new(self.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_success_with_data() {
        let result = BlobBuilder::new().with_data(vec![1, 2, 3, 4]).build();
        assert!(result.is_ok(), "BlobBuilder should succeed with valid data");
    }

    #[test]
    fn test_build_returns_blob_with_correct_data() {
        let data = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let blob = BlobBuilder::new()
            .with_data(data.clone())
            .build()
            .expect("build should succeed");
        assert_eq!(blob.data(), data.as_slice());
    }
}
