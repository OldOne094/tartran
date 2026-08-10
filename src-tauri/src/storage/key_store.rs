#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use std::sync::Mutex;

pub trait KeyStore: Send + Sync {
    fn has(&self, id: &str) -> bool;
    fn set(&self, id: &str, plain: &str) -> Result<(), KeyStoreError>;
    fn get(&self, id: &str) -> Option<String>;
    fn delete(&self, id: &str);
}
#[derive(Debug)]
pub enum KeyStoreError {
    Unavailable,
}

pub struct OsKeyring {
    service: String,
}

impl OsKeyring {
    pub fn new(service: &str) -> Self {
        OsKeyring {
            service: service.to_string(),
        }
    }

    fn entry(&self, id: &str) -> Result<keyring::Entry, KeyStoreError> {
        keyring::Entry::new(&self.service, id).map_err(|_| KeyStoreError::Unavailable)
    }
}

impl KeyStore for OsKeyring {
    fn has(&self, id: &str) -> bool {
        self.get(id).is_some()
    }

    fn set(&self, id: &str, plain: &str) -> Result<(), KeyStoreError> {
        let entry = self.entry(id)?;
        entry.set_password(plain).map_err(|_| KeyStoreError::Unavailable)
    }

    fn get(&self, id: &str) -> Option<String> {
        let entry = self.entry(id).ok()?;
        entry.get_password().ok()
    }

    fn delete(&self, id: &str) {
        if let Ok(entry) = self.entry(id) {
            let _ = entry.delete_credential();
        }
    }
}

#[cfg(test)]
pub struct MemoryKeyStore {
    map: Mutex<HashMap<String, String>>,
}

#[cfg(test)]
impl MemoryKeyStore {
    pub fn new() -> Self {
        MemoryKeyStore {
            map: Mutex::new(HashMap::new()),
        }
    }
}

#[cfg(test)]
impl KeyStore for MemoryKeyStore {
    fn has(&self, id: &str) -> bool {
        self.map.lock().unwrap().contains_key(id)
    }

    fn set(&self, id: &str, plain: &str) -> Result<(), KeyStoreError> {
        self.map.lock().unwrap().insert(id.to_string(), plain.to_string());
        Ok(())
    }

    fn get(&self, id: &str) -> Option<String> {
        self.map.lock().unwrap().get(id).cloned()
    }

    fn delete(&self, id: &str) {
        self.map.lock().unwrap().remove(id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_store_roundtrip() {
        let store = MemoryKeyStore::new();
        assert!(!store.has("default"));
        store.set("default", "AIzaSySecretKey123").unwrap();
        assert!(store.has("default"));
        assert_eq!(store.get("default").unwrap(), "AIzaSySecretKey123");
        store.delete("default");
        assert!(!store.has("default"));
        assert_eq!(store.get("default"), None);
    }
}
