use crate::codec::Encoder;
use crate::domain::blob::Blob;
use crate::domain::hash::Hash;
use crate::domain::tree::{EntryKind, Tree, TreeEntry};
use crate::error::VctrlError;
use crate::hashing::Hasher;
use crate::storage::traits::ObjectStore;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PosItem {
    pub sku: String,
    pub name: String,
    pub qty: u32,
    pub price: u64,
    pub subtotal: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PosTransaction {
    pub transaction_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub cashier_id: String,
    pub items: Vec<PosItem>,
    pub total: u64,
    pub payment_method: String,
}

impl PosTransaction {
    pub fn to_json_bytes(&self) -> Result<Vec<u8>, VctrlError> {
        serde_json::to_vec(self).map_err(|e| VctrlError::Serialization(e.to_string()))
    }
}

pub fn build_transaction_tree(
    date: NaiveDate,
    transactions: &[(String, Vec<u8>)],
    hasher: &dyn Hasher,
    store: &mut dyn ObjectStore,
) -> Result<Tree, VctrlError> {
    let year = date.format("%Y").to_string();
    let month = date.format("%m").to_string();
    let day = date.format("%d").to_string();

    let mut day_entries = Vec::new();
    for (tx_id, json_bytes) in transactions {
        let blob = Blob::new(json_bytes.clone());
        let blob_hash = hasher.hash_blob(blob.as_bytes());
        store.put(&blob_hash, &crate::domain::object::Object::Blob(blob))?;
        let entry =
            TreeEntry::new(tx_id.clone(), EntryKind::Blob, blob_hash).map_err(VctrlError::Tree)?;
        day_entries.push(entry);
    }

    let day_tree = Tree::new(day_entries).map_err(VctrlError::Tree)?;
    let day_hash = store_tree(&day_tree, hasher, store)?;

    let day_entry = TreeEntry::new(day, EntryKind::Tree, day_hash).map_err(VctrlError::Tree)?;
    let month_tree = Tree::new(vec![day_entry]).map_err(VctrlError::Tree)?;
    let month_hash = store_tree(&month_tree, hasher, store)?;

    let month_entry =
        TreeEntry::new(month, EntryKind::Tree, month_hash).map_err(VctrlError::Tree)?;
    let year_tree = Tree::new(vec![month_entry]).map_err(VctrlError::Tree)?;
    let year_hash = store_tree(&year_tree, hasher, store)?;

    let year_entry = TreeEntry::new(year, EntryKind::Tree, year_hash).map_err(VctrlError::Tree)?;
    let transactions_tree = Tree::new(vec![year_entry]).map_err(VctrlError::Tree)?;
    let transactions_hash = store_tree(&transactions_tree, hasher, store)?;

    let root_entry = TreeEntry::new(
        "transactions".to_string(),
        EntryKind::Tree,
        transactions_hash,
    )
    .map_err(VctrlError::Tree)?;
    Tree::new(vec![root_entry]).map_err(VctrlError::Tree)
}

pub fn build_inventory_tree(
    inventory: &HashMap<String, Vec<u8>>,
    hasher: &dyn Hasher,
    store: &mut dyn ObjectStore,
) -> Result<Tree, VctrlError> {
    let mut entries = Vec::new();
    for (sku, data) in inventory {
        let blob = Blob::new(data.clone());
        let blob_hash = hasher.hash_blob(blob.as_bytes());
        store.put(&blob_hash, &crate::domain::object::Object::Blob(blob))?;
        let entry =
            TreeEntry::new(sku.clone(), EntryKind::Blob, blob_hash).map_err(VctrlError::Tree)?;
        entries.push(entry);
    }
    Tree::new(entries).map_err(VctrlError::Tree)
}

fn store_tree(
    tree: &Tree,
    hasher: &dyn Hasher,
    store: &mut dyn ObjectStore,
) -> Result<Hash, VctrlError> {
    let mut buf = Vec::new();
    let encoder = crate::codec::BinaryEncoder;
    encoder.encode_tree(tree, &mut buf)?;
    let hash = hasher.hash_tree_encoded(&buf);
    store.put(&hash, &crate::domain::object::Object::Tree(tree.clone()))?;
    Ok(hash)
}
