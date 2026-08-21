use std::io::Read;

use crate::errors::VctrlError;

pub trait PackWriter: Send + Sync {
    type ObjectId: Send + Sync;

    fn write_object(&mut self, id: &Self::ObjectId, data: &[u8]) -> Result<(), VctrlError>;
    fn finish(&mut self) -> Result<(), VctrlError>;
}

pub trait PackReader: Send + Sync {
    type ObjectId: Send + Sync;

    fn read_object(&self, id: &Self::ObjectId) -> Result<Box<dyn Read + Send + '_>, VctrlError>;
}
