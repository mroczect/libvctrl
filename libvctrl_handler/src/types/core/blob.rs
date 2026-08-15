#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Blob {
    data: Vec<u8>,
}

impl Blob {
    #[allow(clippy::missing_const_for_fn)]
    #[must_use]
    pub fn new(data: Vec<u8>) -> Self {
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
