use std::{collections::HashMap, sync::RwLock};

#[derive(Default)]
pub struct Store {
    inner: RwLock<HashMap<String, String>>,
}

impl Store {
    pub fn get(&self, key: &str) -> Option<String> {
        self.inner.read().unwrap().get(key).cloned()
    }

    pub fn set(&self, key: String, value: String) {
        self.inner.write().unwrap().insert(key, value);
    }
}
