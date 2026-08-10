use libvctrl_handler::Blob;

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

    #[must_use]
    pub fn build(self) -> Blob {
        Blob::new(self.data)
    }
}
