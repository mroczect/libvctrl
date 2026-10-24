use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;
use std::io;

use libvctrl_handler::{ConfigStore, VctrlError};

#[derive(Debug, Clone)]
pub struct MemoryConfigStore {
    values: BTreeMap<String, String>,
}

impl Default for MemoryConfigStore {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryConfigStore {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            values: BTreeMap::new(),
        }
    }

    fn make_key(section: &str, name: &str) -> String {
        format!("{section}.{name}")
    }

    pub fn load_from<R: io::Read + Send>(&mut self, mut reader: R) -> Result<(), VctrlError> {
        let mut content = String::new();
        let _ = reader
            .read_to_string(&mut content)
            .map_err(VctrlError::from_io)?;

        let mut current_section = String::new();

        for line in content.lines() {
            let line = line.trim();

            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }

            if let Some(rest) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                let rest = rest.trim();
                current_section = rest.to_string();
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = Self::strip_quotes(value.trim());
                let full_key = Self::make_key(&current_section, key);
                let _ = self.values.insert(full_key, value.to_string());
            }
        }

        Ok(())
    }

    pub fn save_to<W: io::Write + Send>(&self, mut writer: W) -> Result<(), VctrlError> {
        let mut sections: BTreeMap<&str, Vec<(&str, &str)>> = BTreeMap::new();

        for (key, value) in &self.values {
            if let Some((section, name)) = key.split_once('.') {
                sections.entry(section).or_default().push((name, value));
            }
        }

        for (section, entries) in &sections {
            writeln!(writer, "[{section}]").map_err(VctrlError::from_io)?;
            for (name, value) in entries {
                writeln!(writer, "\t{name} = {value}").map_err(VctrlError::from_io)?;
            }
            writeln!(writer).map_err(VctrlError::from_io)?;
        }

        Ok(())
    }

    fn strip_quotes(s: &str) -> &str {
        s.strip_prefix('"')
            .and_then(|s| s.strip_suffix('"'))
            .unwrap_or(s)
    }

    fn parse_bool(s: &str) -> Option<bool> {
        match s.to_lowercase().as_str() {
            "true" | "yes" | "on" | "1" => Some(true),
            "false" | "no" | "off" | "0" => Some(false),
            _ => None,
        }
    }

    #[must_use]
    pub fn get_raw(&self, section: &str, name: &str) -> Option<&str> {
        let key = Self::make_key(section, name);
        self.values.get(&key).map(String::as_str)
    }

    pub fn set_raw(&mut self, section: &str, name: &str, value: impl Into<String>) {
        let key = Self::make_key(section, name);
        let _ = self.values.insert(key, value.into());
    }

    pub fn remove(&mut self, section: &str, name: &str) -> bool {
        let key = Self::make_key(section, name);
        self.values.remove(&key).is_some()
    }

    #[must_use]
    pub fn keys(&self, section: &str) -> Vec<&str> {
        let prefix = format!("{section}.");
        self.values
            .keys()
            .filter_map(|k| k.strip_prefix(&prefix))
            .collect()
    }

    #[must_use]
    pub fn sections(&self) -> Vec<&str> {
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        for key in self.values.keys() {
            if let Some(section) = key.split_once('.').map(|(s, _)| s) {
                let _ = seen.insert(section);
            }
        }
        seen.into_iter().collect()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn clear(&mut self) {
        self.values.clear();
    }
}

impl ConfigStore for MemoryConfigStore {
    fn get_string(&self, section: &str, name: &str) -> Result<Option<String>, VctrlError> {
        Ok(self.get_raw(section, name).map(str::to_owned))
    }

    fn set_string(&mut self, section: &str, name: &str, value: &str) -> Result<(), VctrlError> {
        self.set_raw(section, name, value);
        Ok(())
    }

    fn get_bool(&self, section: &str, name: &str) -> Result<Option<bool>, VctrlError> {
        Ok(self.get_raw(section, name).and_then(Self::parse_bool))
    }

    fn set_bool(&mut self, section: &str, name: &str, value: bool) -> Result<(), VctrlError> {
        let s = if value { "true" } else { "false" };
        self.set_raw(section, name, s);
        Ok(())
    }

    fn exists(&self, section: &str, name: &str) -> Result<bool, VctrlError> {
        let key = Self::make_key(section, name);
        Ok(self.values.contains_key(&key))
    }

    fn remove(&mut self, section: &str, name: &str) -> Result<(), VctrlError> {
        let _ = Self::remove(self, section, name);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_missing_returns_none() -> Result<(), VctrlError> {
        let store = MemoryConfigStore::new();
        assert_eq!(store.get_string("core", "missing")?, None);
        Ok(())
    }

    #[test]
    fn set_and_get_string() -> Result<(), VctrlError> {
        let mut store = MemoryConfigStore::new();
        store.set_string("user", "name", "Alice")?;

        assert_eq!(store.get_string("user", "name")?, Some("Alice".to_string()));
        Ok(())
    }

    #[test]
    fn set_and_get_bool() -> Result<(), VctrlError> {
        let mut store = MemoryConfigStore::new();
        store.set_bool("core", "autocrlf", true)?;

        assert_eq!(store.get_bool("core", "autocrlf")?, Some(true));

        store.set_bool("core", "autocrlf", false)?;
        assert_eq!(store.get_bool("core", "autocrlf")?, Some(false));
        Ok(())
    }

    #[test]
    fn get_bool_parses_various_formats() -> Result<(), VctrlError> {
        let mut store = MemoryConfigStore::new();

        for val in ["true", "TRUE", "yes", "YES", "on", "ON", "1"] {
            store.set_string("test", "val", val)?;
            assert_eq!(store.get_bool("test", "val")?, Some(true));
        }

        for val in ["false", "FALSE", "no", "NO", "off", "OFF", "0"] {
            store.set_string("test", "val", val)?;
            assert_eq!(store.get_bool("test", "val")?, Some(false));
        }

        store.set_string("test", "val", "maybe")?;
        assert_eq!(store.get_bool("test", "val")?, None);
        Ok(())
    }

    #[test]
    fn exists_check() -> Result<(), VctrlError> {
        let mut store = MemoryConfigStore::new();
        assert!(!store.exists("core", "bare")?);

        store.set_string("core", "bare", "true")?;
        assert!(store.exists("core", "bare")?);
        Ok(())
    }

    #[test]
    fn different_sections_independent() -> Result<(), VctrlError> {
        let mut store = MemoryConfigStore::new();
        store.set_string("user", "name", "Alice")?;
        store.set_string("core", "name", "different")?;

        assert_eq!(store.get_string("user", "name")?, Some("Alice".to_string()));
        assert_eq!(
            store.get_string("core", "name")?,
            Some("different".to_string())
        );
        Ok(())
    }

    #[test]
    fn remove_entry() -> Result<(), VctrlError> {
        let mut store = MemoryConfigStore::new();
        store.set_string("test", "key", "value")?;
        assert!(store.exists("test", "key")?);

        assert!(MemoryConfigStore::remove(&mut store, "test", "key"));
        assert!(!store.exists("test", "key")?);
        assert!(!MemoryConfigStore::remove(&mut store, "test", "key"));
        Ok(())
    }

    #[test]
    fn keys_listing() -> Result<(), VctrlError> {
        let mut store = MemoryConfigStore::new();
        store.set_string("core", "bare", "true")?;
        store.set_string("core", "autocrlf", "false")?;
        store.set_string("user", "name", "Alice")?;

        let mut core_keys = store.keys("core");
        core_keys.sort_unstable();
        assert_eq!(core_keys, vec!["autocrlf", "bare"]);

        let user_keys = store.keys("user");
        assert_eq!(user_keys, vec!["name"]);
        Ok(())
    }

    #[test]
    fn sections_listing() -> Result<(), VctrlError> {
        let mut store = MemoryConfigStore::new();
        store.set_string("user", "name", "A")?;
        store.set_string("core", "bare", "true")?;
        store.set_string("remote", "url", "http://x")?;

        let mut sections = store.sections();
        sections.sort_unstable();
        assert_eq!(sections, vec!["core", "remote", "user"]);
        Ok(())
    }

    #[test]
    fn len_and_empty() -> Result<(), VctrlError> {
        let mut store = MemoryConfigStore::new();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);

        store.set_string("a", "b", "c")?;
        assert!(!store.is_empty());
        assert_eq!(store.len(), 1);
        Ok(())
    }

    #[test]
    fn clear_all() -> Result<(), VctrlError> {
        let mut store = MemoryConfigStore::new();
        store.set_string("a", "b", "c")?;
        store.set_string("d", "e", "f")?;

        store.clear();
        assert!(store.is_empty());
        Ok(())
    }

    #[test]
    fn save_and_load_roundtrip() -> Result<(), VctrlError> {
        let mut store1 = MemoryConfigStore::new();
        store1.set_string("user", "name", "Bob")?;
        store1.set_string("core", "bare", "false")?;
        store1.set_bool("core", "autocrlf", true)?;

        let mut buf = Vec::new();
        store1.save_to(&mut buf)?;

        let mut store2 = MemoryConfigStore::new();
        store2.load_from(buf.as_slice())?;

        assert_eq!(store2.get_string("user", "name")?, Some("Bob".to_string()));
        assert_eq!(
            store2.get_string("core", "bare")?,
            Some("false".to_string())
        );
        assert_eq!(store2.get_bool("core", "autocrlf")?, Some(true));
        Ok(())
    }

    #[test]
    fn load_ignores_comments_and_empty() -> Result<(), VctrlError> {
        let config = r"
# This is a comment
; This is also a comment

[core]
    bare = true
    # inline comment
    autocrlf = false
";
        let mut store = MemoryConfigStore::new();
        store.load_from(config.as_bytes())?;

        assert_eq!(store.len(), 2);
        assert_eq!(store.get_string("core", "bare")?, Some("true".to_string()));
        assert_eq!(
            store.get_string("core", "autocrlf")?,
            Some("false".to_string())
        );
        Ok(())
    }

    #[test]
    fn load_strips_quotes() -> Result<(), VctrlError> {
        let config = r#"
[user]
    name = "Alice"
    email = 'alice@example.com'
"#;
        let mut store = MemoryConfigStore::new();
        store.load_from(config.as_bytes())?;

        assert_eq!(store.get_string("user", "name")?, Some("Alice".to_string()));

        assert_eq!(
            store.get_string("user", "email")?,
            Some("'alice@example.com'".to_string())
        );
        Ok(())
    }

    #[test]
    fn clone_preserves_data() -> Result<(), VctrlError> {
        let mut store = MemoryConfigStore::new();
        store.set_string("test", "key", "value")?;

        let cloned = store.clone();
        assert_eq!(cloned.get_string("test", "key")?, Some("value".to_string()));
        Ok(())
    }
}
