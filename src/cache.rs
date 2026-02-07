//! Embedded cache module using sled for persistent caching
//! 
//! Designed for offline/air-gapped environments on low-power computers.
//! Caches query results to speed up dashboard loading.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Cache entry with TTL support
#[derive(Serialize, Deserialize)]
struct CacheEntry {
    data: Vec<u8>,
    created_at: u64,
    ttl_seconds: u64,
}

impl CacheEntry {
    fn new(data: Vec<u8>, ttl: Duration) -> Self {
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            data,
            created_at,
            ttl_seconds: ttl.as_secs(),
        }
    }
    
    fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now > self.created_at + self.ttl_seconds
    }
}

/// Persistent cache using sled embedded database
pub struct Cache {
    db: sled::Db,
    default_ttl: Duration,
}

impl Cache {
    /// Open or create a cache at the specified path
    pub fn open(root: &Path) -> Result<Self> {
        let cache_path = root.join("cache");
        let db = sled::open(&cache_path)
            .context("Failed to open sled cache database")?;
        
        Ok(Self {
            db,
            default_ttl: Duration::from_secs(300), // 5 minutes default
        })
    }
    
    /// Set default TTL for cache entries
    pub fn set_default_ttl(&mut self, ttl: Duration) {
        self.default_ttl = ttl;
    }
    
    /// Get a value from cache
    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        let raw = self.db.get(key.as_bytes()).ok()??;
        let entry: CacheEntry = serde_json::from_slice(&raw).ok()?;
        
        if entry.is_expired() {
            // Remove expired entry
            let _ = self.db.remove(key.as_bytes());
            return None;
        }
        
        Some(entry.data)
    }
    
    /// Get a string value from cache
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.get(key).and_then(|data| String::from_utf8(data).ok())
    }
    
    /// Set a value in cache with default TTL
    pub fn set(&self, key: &str, value: &[u8]) -> Result<()> {
        self.set_with_ttl(key, value, self.default_ttl)
    }
    
    /// Set a value with custom TTL
    pub fn set_with_ttl(&self, key: &str, value: &[u8], ttl: Duration) -> Result<()> {
        let entry = CacheEntry::new(value.to_vec(), ttl);
        let serialized = serde_json::to_vec(&entry)?;
        self.db.insert(key.as_bytes(), serialized)?;
        self.db.flush()?;
        Ok(())
    }
    
    /// Set a string value in cache
    pub fn set_string(&self, key: &str, value: &str) -> Result<()> {
        self.set(key, value.as_bytes())
    }
    
    /// Remove a key from cache
    pub fn remove(&self, key: &str) -> Result<()> {
        self.db.remove(key.as_bytes())?;
        Ok(())
    }
    
    /// Clear all cache entries
    pub fn clear(&self) -> Result<()> {
        self.db.clear()?;
        self.db.flush()?;
        Ok(())
    }
    
    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            entries: self.db.len(),
            size_bytes: self.db.size_on_disk().unwrap_or(0),
        }
    }
}

/// Cache statistics
pub struct CacheStats {
    pub entries: usize,
    pub size_bytes: u64,
}

impl std::fmt::Display for CacheStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cache: {} entries, {:.2} KB on disk",
            self.entries,
            self.size_bytes as f64 / 1024.0
        )
    }
}

/// Convenience function to create cache key from query parameters
pub fn make_cache_key(prefix: &str, params: &[(&str, &str)]) -> String {
    let mut key = prefix.to_string();
    for (k, v) in params {
        key.push(':');
        key.push_str(k);
        key.push('=');
        key.push_str(v);
    }
    key
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_cache_basic() {
        let dir = tempdir().unwrap();
        let cache = Cache::open(dir.path()).unwrap();
        
        cache.set_string("test_key", "test_value").unwrap();
        assert_eq!(cache.get_string("test_key"), Some("test_value".to_string()));
    }
    
    #[test]
    fn test_cache_expiry() {
        let dir = tempdir().unwrap();
        let cache = Cache::open(dir.path()).unwrap();
        
        // Set with 0 second TTL (immediately expired)
        cache.set_with_ttl("expired", b"value", Duration::from_secs(0)).unwrap();
        std::thread::sleep(Duration::from_secs(2));
        assert!(cache.get("expired").is_none());
    }
}
