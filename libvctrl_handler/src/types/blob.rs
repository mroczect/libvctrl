#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Blob {
    data: Vec<u8>,
}

impl Blob {
    #[must_use]
    pub const fn new(data: Vec<u8>) -> Self {
        Self { data }
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
