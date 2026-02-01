use parking_lot::RwLock;
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

#[derive(Default)]
pub struct Store {
    inner: RwLock<HashMap<String, StoreEntry>>,
}

#[derive(Debug)]
struct StoreEntry {
    value: String,
    expires_at: Option<Instant>,
}

impl Store {
    pub fn get(&self, key: &str) -> Option<String> {
        {
            let inner = self.inner.read();
            if let Some(entry) = inner.get(key) {
                match entry.expires_at {
                    Some(exp) if exp < Instant::now() => {}
                    _ => return Some(entry.value.clone()),
                }
            } else {
                return None;
            }
        }

        let mut inner = self.inner.write();
        if let Some(entry) = inner.get(key) {
            if entry
                .expires_at
                .is_some_and(|expiry| expiry < Instant::now())
            {
                inner.remove(key);
            }
        }
        None
    }

    pub fn set(&self, key: String, value: String, expiry: Option<Duration>) {
        let expires_at = expiry.map(|e| Instant::now() + e);
        let entry = StoreEntry { value, expires_at };
        self.inner.write().insert(key, entry);
    }
}
