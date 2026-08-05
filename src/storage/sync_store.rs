use crate::domain::hash::Hash;
use crate::domain::object::Object;
use crate::error::VctrlError;
use crate::storage::traits::{ObjectStore, RefStore};
use std::sync::{Arc, Mutex};

pub trait SyncObjectStore: Send + Sync {
    fn put(&self, hash: &Hash, obj: &Object) -> Result<(), VctrlError>;
    fn get(&self, hash: &Hash) -> Result<Option<Object>, VctrlError>;
    fn exists(&self, hash: &Hash) -> Result<bool, VctrlError>;
}

pub trait SyncRefStore: Send + Sync {
    fn set_ref(&self, name: &str, hash: &Hash) -> Result<(), VctrlError>;
    fn get_ref(&self, name: &str) -> Result<Option<Hash>, VctrlError>;
    fn delete_ref(&self, name: &str) -> Result<(), VctrlError>;
    fn set_head(&self, target: &str) -> Result<(), VctrlError>;
    fn head(&self) -> Result<Option<Hash>, VctrlError>;
    fn head_ref_name(&self) -> Result<Option<String>, VctrlError>;
    fn list_refs(&self, prefix: &str) -> Result<Vec<String>, VctrlError>;
}

pub struct SyncAdapter<S> {
    inner: Arc<Mutex<S>>,
}

impl<S> SyncAdapter<S> {
    pub fn new(inner: S) -> Self {
        Self {
            inner: Arc::new(Mutex::new(inner)),
        }
    }
}

impl<S: ObjectStore + Send + 'static> SyncObjectStore for SyncAdapter<S> {
    fn put(&self, hash: &Hash, obj: &Object) -> Result<(), VctrlError> {
        self.inner.lock().unwrap().put(hash, obj)
    }
    fn get(&self, hash: &Hash) -> Result<Option<Object>, VctrlError> {
        self.inner.lock().unwrap().get(hash)
    }
    fn exists(&self, hash: &Hash) -> Result<bool, VctrlError> {
        self.inner.lock().unwrap().exists(hash)
    }
}

impl<S: RefStore + Send + 'static> SyncRefStore for SyncAdapter<S> {
    fn set_ref(&self, name: &str, hash: &Hash) -> Result<(), VctrlError> {
        self.inner.lock().unwrap().set_ref(name, hash)
    }
    fn get_ref(&self, name: &str) -> Result<Option<Hash>, VctrlError> {
        self.inner.lock().unwrap().get_ref(name)
    }
    fn delete_ref(&self, name: &str) -> Result<(), VctrlError> {
        self.inner.lock().unwrap().delete_ref(name)
    }
    fn set_head(&self, target: &str) -> Result<(), VctrlError> {
        self.inner.lock().unwrap().set_head(target)
    }
    fn head(&self) -> Result<Option<Hash>, VctrlError> {
        self.inner.lock().unwrap().head()
    }
    fn head_ref_name(&self) -> Result<Option<String>, VctrlError> {
        self.inner.lock().unwrap().head_ref_name()
    }
    fn list_refs(&self, prefix: &str) -> Result<Vec<String>, VctrlError> {
        self.inner.lock().unwrap().list_refs(prefix)
    }
}
