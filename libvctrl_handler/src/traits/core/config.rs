use crate::VctrlError;

pub trait ConfigStore {
    fn get_string(&self, section: &str, key: &str) -> Result<Option<String>, VctrlError>;
}
