#[cfg(test)]
use libvctrl as _;
extern crate alloc;
use alloc::sync::Arc;
use core::sync::atomic::{AtomicBool, Ordering};
use monumentum_core as _;
use std::io::Cursor;
use std::sync::Mutex;

use libvctrl_handler::{Hash, ObjectStore, RefStore, VctrlError};
use monumentum_handler::{ColumnDef, DataType, DbError, Row, StorageEngine, TableSchema, Value};

pub const OBJECTS_TABLE: &str = "_vctrl_objects";
pub const REFS_TABLE: &str = "_vctrl_refs";

pub struct MonumentumStore<E: StorageEngine> {
    engine: Arc<Mutex<E>>,
    initialized: Arc<AtomicBool>,
}

impl<E: StorageEngine> core::fmt::Debug for MonumentumStore<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MonumentumStore")
            .field("initialized", &self.initialized.load(Ordering::Acquire))
            .finish_non_exhaustive()
    }
}

impl<E: StorageEngine> Clone for MonumentumStore<E> {
    fn clone(&self) -> Self {
        Self {
            engine: Arc::clone(&self.engine),
            initialized: Arc::clone(&self.initialized),
        }
    }
}

impl<E: StorageEngine> MonumentumStore<E> {
    pub fn new(engine: E) -> Self {
        Self {
            engine: Arc::new(Mutex::new(engine)),
            initialized: Arc::new(AtomicBool::new(false)),
        }
    }

    fn with_engine<R>(&self, f: impl FnOnce(&mut E) -> R) -> R {
        let mut guard = self
            .engine
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        f(&mut guard)
    }

    fn ensure_tables(&self) -> Result<(), VctrlError> {
        if self.initialized.load(Ordering::Acquire) {
            return Ok(());
        }

        let objects_schema = TableSchema::try_new(
            OBJECTS_TABLE,
            vec![
                {
                    let mut c = ColumnDef::new("hash", DataType::Blob);
                    c.set_primary_key(true);
                    c
                },
                ColumnDef::new("data", DataType::Blob),
            ],
        )
        .map_err(to_vctrl)?;

        let refs_schema = TableSchema::try_new(
            REFS_TABLE,
            vec![
                {
                    let mut c = ColumnDef::new("name", DataType::Text);
                    c.set_primary_key(true);
                    c
                },
                ColumnDef::new("hash", DataType::Blob),
            ],
        )
        .map_err(to_vctrl)?;

        self.with_engine(|engine| {
            let _ = engine.create_table(objects_schema);
            let _ = engine.create_table(refs_schema);
        });

        self.initialized.store(true, Ordering::Release);
        Ok(())
    }
}

impl<E: StorageEngine + Send + 'static> ObjectStore for MonumentumStore<E> {
    fn put(&mut self, hash: &Hash, data: &[u8]) -> Result<(), VctrlError> {
        self.ensure_tables()?;

        let key = Value::try_from(hash.as_bytes().to_vec()).map_err(to_vctrl)?;
        let already_exists = self
            .with_engine(|engine| engine.get_row_by_key(OBJECTS_TABLE, &key).map_err(to_vctrl))?
            .is_some();
        if already_exists {
            return Ok(());
        }

        let row = Row::new(vec![
            Value::try_from(hash.as_bytes().to_vec()).map_err(to_vctrl)?,
            Value::try_from(data.to_vec()).map_err(to_vctrl)?,
        ]);
        self.with_engine(|engine| engine.insert_row(OBJECTS_TABLE, &row).map_err(to_vctrl))?;
        Ok(())
    }

    fn get(&self, hash: &Hash) -> Result<Box<dyn std::io::Read + Send + '_>, VctrlError> {
        self.ensure_tables()?;
        let key = Value::try_from(hash.as_bytes().to_vec()).map_err(to_vctrl)?;
        let row = self
            .with_engine(|engine| engine.get_row_by_key(OBJECTS_TABLE, &key).map_err(to_vctrl))?
            .ok_or(VctrlError::ObjectNotFound(*hash))?;

        let bytes = row
            .get(1)
            .and_then(Value::as_blob)
            .map(|b| b.as_slice().to_vec())
            .ok_or_else(|| VctrlError::CorruptedData("missing data column".into()))?;

        Ok(Box::new(Cursor::new(bytes)))
    }

    fn delete(&mut self, _hash: &Hash) -> Result<(), VctrlError> {
        Ok(())
    }

    fn exists(&self, hash: &Hash) -> Result<bool, VctrlError> {
        self.ensure_tables()?;
        let key = Value::try_from(hash.as_bytes().to_vec()).map_err(to_vctrl)?;
        let row = self
            .with_engine(|engine| engine.get_row_by_key(OBJECTS_TABLE, &key).map_err(to_vctrl))?;
        Ok(row.is_some())
    }
}

impl<E: StorageEngine + Send + 'static> RefStore for MonumentumStore<E> {
    fn set_ref(&mut self, name: &str, hash: &Hash) -> Result<(), VctrlError> {
        self.ensure_tables()?;
        let key = Value::try_from(name.to_string()).map_err(to_vctrl)?;
        let new_row = Row::new(vec![
            Value::try_from(name.to_string()).map_err(to_vctrl)?,
            Value::try_from(hash.as_bytes().to_vec()).map_err(to_vctrl)?,
        ]);

        self.with_engine(|engine| -> Result<(), VctrlError> {
            let existing = engine.get_row_by_key(REFS_TABLE, &key).map_err(to_vctrl)?;
            if existing.is_some() {
                let mut all = engine.get_all_rows(REFS_TABLE).map_err(to_vctrl)?;
                for row in &mut all {
                    if row.get(0).and_then(Value::as_str) == Some(name) {
                        *row = new_row.clone();
                    }
                }
                engine.replace_rows(REFS_TABLE, all).map_err(to_vctrl)?;
            } else {
                engine.insert_row(REFS_TABLE, &new_row).map_err(to_vctrl)?;
            }
            Ok(())
        })?;
        Ok(())
    }

    fn get_ref(&self, name: &str) -> Result<Hash, VctrlError> {
        self.ensure_tables()?;
        let key = Value::try_from(name.to_string()).map_err(to_vctrl)?;
        let row = self
            .with_engine(|engine| engine.get_row_by_key(REFS_TABLE, &key).map_err(to_vctrl))?
            .ok_or_else(|| VctrlError::RefNotFound(name.into()))?;

        let bytes = row
            .get(1)
            .and_then(Value::as_blob)
            .map(|b| b.as_slice().to_vec())
            .ok_or_else(|| VctrlError::CorruptedData("missing hash column".into()))?;

        Hash::from_bytes(&bytes)
    }

    fn delete_ref(&mut self, name: &str) -> Result<(), VctrlError> {
        self.ensure_tables()?;
        self.with_engine(|engine| -> Result<(), VctrlError> {
            let mut all = engine.get_all_rows(REFS_TABLE).map_err(to_vctrl)?;
            all.retain(|row| row.get(0).and_then(Value::as_str) != Some(name));
            engine.replace_rows(REFS_TABLE, all).map_err(to_vctrl)
        })?;
        Ok(())
    }

    fn list_refs(&self) -> Result<Vec<String>, VctrlError> {
        self.ensure_tables()?;
        let mut names: Vec<String> = self
            .with_engine(|engine| engine.get_all_rows(REFS_TABLE).map_err(to_vctrl))?
            .into_iter()
            .filter_map(|row| row.get(0).and_then(Value::as_str).map(str::to_owned))
            .collect();
        names.sort();
        Ok(names)
    }
}

// DbError is #[non_exhaustive]: the wildcard is required for forward
// compatibility with variants added in future monumentum_handler versions.
#[allow(clippy::wildcard_enum_match_arm)]
fn to_vctrl(e: DbError) -> VctrlError {
    match e {
        DbError::Io(io) => VctrlError::IoError(io),
        other => VctrlError::Other(format!("monumentum: {other}")),
    }
}
