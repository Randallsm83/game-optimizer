//! Local caching layer for API responses

use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use crate::config::Config;

/// Cache entry with TTL
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct CacheEntry<T> {
    data: T,
    created_at: u64, // Unix timestamp
    ttl_secs: u64,
}

/// Simple file-based cache
pub struct Cache {
    cache_dir: PathBuf,
    default_ttl: Duration,
}

impl Cache {
    /// Create a new cache with default settings
    pub fn new() -> Result<Self> {
        let cache_dir = Config::cache_dir();
        std::fs::create_dir_all(&cache_dir)?;
        
        Ok(Self {
            cache_dir,
            default_ttl: Duration::from_secs(24 * 60 * 60), // 24 hours
        })
    }

    /// Create cache with custom TTL
    pub fn with_ttl(ttl: Duration) -> Result<Self> {
        let mut cache = Self::new()?;
        cache.default_ttl = ttl;
        Ok(cache)
    }

    /// Get a cached value
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let path = self.key_path(key);
        
        let content = std::fs::read_to_string(&path).ok()?;
        let entry: CacheEntry<T> = serde_json::from_str(&content).ok()?;
        
        // Check if expired
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .ok()?
            .as_secs();
        
        if now > entry.created_at + entry.ttl_secs {
            // Expired, remove the file
            let _ = std::fs::remove_file(&path);
            return None;
        }
        
        Some(entry.data)
    }

    /// Set a cached value with default TTL
    pub fn set<T: Serialize>(&self, key: &str, value: &T) -> Result<()> {
        self.set_with_ttl(key, value, self.default_ttl)
    }

    /// Set a cached value with custom TTL
    pub fn set_with_ttl<T: Serialize>(&self, key: &str, value: &T, ttl: Duration) -> Result<()> {
        let path = self.key_path(key);
        
        let entry = CacheEntry {
            data: value,
            created_at: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_secs(),
            ttl_secs: ttl.as_secs(),
        };
        
        let content = serde_json::to_string(&entry)?;
        std::fs::write(path, content)?;
        
        Ok(())
    }

    /// Remove a cached value
    pub fn remove(&self, key: &str) -> Result<()> {
        let path = self.key_path(key);
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }

    /// Clear all cached values
    pub fn clear(&self) -> Result<()> {
        if self.cache_dir.exists() {
            for entry in std::fs::read_dir(&self.cache_dir)? {
                let entry = entry?;
                if entry.path().extension().map(|e| e == "json").unwrap_or(false) {
                    std::fs::remove_file(entry.path())?;
                }
            }
        }
        Ok(())
    }

    /// Get cache statistics
    pub fn stats(&self) -> Result<CacheStats> {
        let mut count = 0;
        let mut total_size = 0;
        let mut expired = 0;

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs();

        if self.cache_dir.exists() {
            for entry in std::fs::read_dir(&self.cache_dir)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.extension().map(|e| e == "json").unwrap_or(false) {
                    count += 1;
                    total_size += entry.metadata().map(|m| m.len()).unwrap_or(0);
                    
                    // Check if expired
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(cache_entry) = serde_json::from_str::<CacheEntry<serde_json::Value>>(&content) {
                            if now > cache_entry.created_at + cache_entry.ttl_secs {
                                expired += 1;
                            }
                        }
                    }
                }
            }
        }

        Ok(CacheStats {
            entries: count,
            expired_entries: expired,
            total_size_bytes: total_size,
        })
    }

    fn key_path(&self, key: &str) -> PathBuf {
        // Sanitize key for filesystem
        let safe_key: String = key
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
            .collect();
        self.cache_dir.join(format!("{}.json", safe_key))
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::new().expect("Failed to create cache")
    }
}

/// Cache statistics
#[derive(Debug)]
pub struct CacheStats {
    pub entries: usize,
    pub expired_entries: usize,
    pub total_size_bytes: u64,
}

impl CacheStats {
    pub fn size_display(&self) -> String {
        const KB: u64 = 1024;
        const MB: u64 = 1024 * 1024;
        
        if self.total_size_bytes >= MB {
            format!("{:.1} MB", self.total_size_bytes as f64 / MB as f64)
        } else if self.total_size_bytes >= KB {
            format!("{:.1} KB", self.total_size_bytes as f64 / KB as f64)
        } else {
            format!("{} bytes", self.total_size_bytes)
        }
    }
}
