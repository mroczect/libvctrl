use crate::errors::VctrlError;

pub trait ConfigStore: Send + Sync {
    fn get_string(&self, section: &str, key: &str) -> Result<Option<String>, VctrlError>;
    fn set_string(&mut self, section: &str, key: &str, value: &str) -> Result<(), VctrlError>;
    fn get_bool(&self, section: &str, key: &str) -> Result<Option<bool>, VctrlError>;
    fn set_bool(&mut self, section: &str, key: &str, value: bool) -> Result<(), VctrlError>;
    fn remove(&mut self, section: &str, key: &str) -> Result<(), VctrlError>;
    fn exists(&self, section: &str, key: &str) -> Result<bool, VctrlError>;
}
