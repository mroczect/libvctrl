use crate::VctrlError;

pub trait PackWriter {
    type ObjectId;

    fn write_object(&mut self, id: &Self::ObjectId, data: &[u8]) -> Result<(), VctrlError>;

    fn finish(&mut self) -> Result<(), VctrlError>;
}

pub trait PackReader {
    type ObjectId;

    fn read_object(&self, id: &Self::ObjectId) -> Result<Vec<u8>, VctrlError>;
}
