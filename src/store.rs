use anyhow::Result;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Represents a stored value with optional expiration
#[derive(Clone, Debug)]
pub struct StoredValue {
    pub data: Vec<u8>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl StoredValue {
    fn is_expired(&self) -> bool {
        if let Some(exp) = self.expires_at {
            Utc::now() > exp
        } else {
            false
        }
    }
}

/// In-memory key-value store with TTL support
#[derive(Clone)]
pub struct KvStore {
    data: Arc<RwLock<HashMap<String, StoredValue>>>,
}

impl KvStore {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let store = self.data.read().await;
        if let Some(value) = store.get(key) {
            if value.is_expired() {
                drop(store);
                self.data.write().await.remove(key);
                return Ok(None);
            }
            Ok(Some(value.data.clone()))
        } else {
            Ok(None)
        }
    }

    pub async fn set(&self, key: String, value: Vec<u8>, ttl_secs: Option<u64>) -> Result<()> {
        let expires_at = ttl_secs.map(|secs| Utc::now() + chrono::Duration::seconds(secs as i64));
        let stored = StoredValue {
            data: value,
            expires_at,
        };
        self.data.write().await.insert(key, stored);
        Ok(())
    }

    pub async fn delete(&self, key: &str) -> Result<bool> {
        Ok(self.data.write().await.remove(key).is_some())
    }

    pub async fn cleanup_expired(&self) {
        let mut store = self.data.write().await;
        store.retain(|_, v| !v.is_expired());
    }
}
