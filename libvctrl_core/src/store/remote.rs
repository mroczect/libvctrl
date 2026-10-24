use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use std::io;

use libvctrl_handler::{
    FetchRefSpec, Hash, PushUpdate, ReflogStore, Remote, RemoteCapabilities, VctrlError,
};

#[derive(Debug, Clone)]
pub struct MemoryRemote {
    name: String,
    url: String,
    fetch_specs: Vec<FetchRefSpec>,
    push_specs: Vec<FetchRefSpec>,
}

impl MemoryRemote {
    pub fn new(name: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            url: url.into(),
            fetch_specs: Vec::new(),
            push_specs: Vec::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn url(&self) -> &str {
        &self.url
    }
}

impl Remote for MemoryRemote {
    fn name(&self) -> &str {
        &self.name
    }

    fn url(&self) -> &str {
        &self.url
    }

    fn pushurl(&self) -> Option<&str> {
        None
    }

    fn set_url(&mut self, url: impl Into<String>) {
        self.url = url.into();
    }

    fn set_pushurl(&mut self, _url: Option<impl Into<String>>) {}

    fn add_fetch(&mut self, spec: FetchRefSpec) -> Result<(), VctrlError> {
        self.fetch_specs.push(spec);
        Ok(())
    }

    fn add_push(&mut self, spec: FetchRefSpec) -> Result<(), VctrlError> {
        self.push_specs.push(spec);
        Ok(())
    }

    fn fetch_specs(&self) -> &[FetchRefSpec] {
        &self.fetch_specs
    }

    fn push_specs(&self) -> &[FetchRefSpec] {
        &self.push_specs
    }

    fn clear_specs(&mut self) {
        self.fetch_specs.clear();
        self.push_specs.clear();
    }

    fn capabilities(&self) -> RemoteCapabilities {
        RemoteCapabilities::empty()
    }

    fn is_connected(&self) -> bool {
        false
    }

    fn connect(&mut self) -> Result<(), VctrlError> {
        Ok(())
    }

    fn disconnect(&mut self) -> Result<(), VctrlError> {
        Ok(())
    }

    fn ls_refs(&self) -> Result<Vec<(String, Hash)>, VctrlError> {
        Ok(Vec::new())
    }

    fn fetch(
        &mut self,
        _refspecs: &[&str],
        _reflog: Option<&mut dyn ReflogStore>,
    ) -> Result<Vec<Hash>, VctrlError> {
        Ok(Vec::new())
    }

    fn push(
        &mut self,
        _updates: &[PushUpdate],
        _reflog: Option<&mut dyn ReflogStore>,
    ) -> Result<Vec<PushUpdate>, VctrlError> {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_remote() {
        let remote = MemoryRemote::new("origin", "https://github.com/user/repo.git");
        assert_eq!(remote.name(), "origin");
        assert_eq!(remote.url(), "https://github.com/user/repo.git");
    }

    #[test]
    fn set_url() {
        let mut remote = MemoryRemote::new("origin", "old-url");
        remote.set_url("new-url");
        assert_eq!(remote.url(), "new-url");
    }

    #[test]
    fn fetch_specs_management() {
        let mut remote = MemoryRemote::new("origin", "url");

        let spec = FetchRefSpec::new("+refs/heads/*:refs/remotes/origin/*").unwrap();
        remote.add_fetch(spec).unwrap();

        assert_eq!(remote.fetch_specs().len(), 1);

        remote.clear_specs();
        assert!(remote.fetch_specs().is_empty());
    }

    #[test]
    fn connect_disconnect() {
        let mut remote = MemoryRemote::new("origin", "url");
        assert!(!remote.is_connected());

        remote.connect().unwrap();
        assert!(!remote.is_connected());

        remote.disconnect().unwrap();
    }

    #[test]
    fn ls_refs_empty() {
        let remote = MemoryRemote::new("origin", "url");
        let refs = remote.ls_refs().unwrap();
        assert!(refs.is_empty());
    }
}
