//! # Persistent Configuration Service
//!
//! ## Rust concepts covered:
//! - `Arc<RwLock<T>>` for shared mutable state across async tasks
//! - Serde for JSON serialization/deserialization
//! - Async file I/O with `tokio::fs`
//! - `#[serde(default)]` for backwards-compatible config
//! - Scoped locks to prevent deadlocks

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// `#[derive(Serialize, Deserialize)]` auto-generates JSON conversion code.
/// `#[serde(default = "...")]` provides a fallback value when the field is
/// missing from the JSON — useful for adding new fields without breaking
/// existing config files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotConfig {
    pub free_games_channel_id: Option<u64>,

    #[serde(default = "default_hour")]
    pub free_games_hour: u32,

    #[serde(default)]
    pub free_games_minute: u32,
}

fn default_hour() -> u32 {
    9
}

/// Manual `Default` implementation when derive isn't flexible enough.
/// `Default::default()` creates a value with sensible defaults.
impl Default for BotConfig {
    fn default() -> Self {
        Self {
            free_games_channel_id: None,
            free_games_hour: 9,
            free_games_minute: 0,
        }
    }
}

/// Thread-safe configuration manager.
///
/// ## `Arc<RwLock<T>>` — the most important concurrency pattern in Rust:
/// - `Arc` (Atomic Reference Counting): allows multiple owners across threads.
///   `.clone()` increments a counter, `Drop` decrements it — zero deallocates.
/// - `RwLock`: allows multiple concurrent readers OR one exclusive writer.
///   More efficient than `Mutex` when reads vastly outnumber writes.
///
/// `#[derive(Clone)]` works because `Arc::clone()` is cheap (counter increment).
#[derive(Clone)]
pub struct ConfigManager {
    config: Arc<RwLock<BotConfig>>,
    path: PathBuf,
}

impl ConfigManager {
    /// Loads config from disk, or creates a default if the file doesn't exist.
    /// `tokio::fs` provides async versions of `std::fs` functions.
    pub async fn load(path: PathBuf) -> Self {
        let config = if path.exists() {
            match tokio::fs::read_to_string(&path).await {
                Ok(content) => {
                    // serde_json::from_str deserializes JSON into a Rust struct.
                    // .unwrap_or_default() falls back to Default on parse error.
                    serde_json::from_str(&content).unwrap_or_default()
                }
                Err(e) => {
                    tracing::warn!("Failed to read config: {}", e);
                    BotConfig::default()
                }
            }
        } else {
            BotConfig::default()
        };

        Self {
            config: Arc::new(RwLock::new(config)),
            path,
        }
    }

    /// `.read().await` acquires a shared read lock.
    /// Multiple tasks can read simultaneously.
    pub async fn save(&self) -> Result<(), std::io::Error> {
        let config = self.config.read().await;

        // `&*config` dereferences the RwLockReadGuard to access the inner value
        let json = serde_json::to_string_pretty(&*config)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        if let Some(parent) = self.path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::write(&self.path, json).await
    }

    /// Returns a clone of the current config.
    /// Clone is necessary because the RwLockReadGuard can't escape this function.
    pub async fn get(&self) -> BotConfig {
        self.config.read().await.clone()
    }

    /// `.write().await` acquires an exclusive write lock.
    /// The lock is scoped with `{ }` and released before calling `save()`,
    /// preventing a deadlock (save() also needs to acquire a read lock).
    pub async fn set_free_games_channel(&self, channel_id: Option<u64>) -> Result<(), std::io::Error> {
        {
            let mut config = self.config.write().await;
            config.free_games_channel_id = channel_id;
        } // Write lock released here
        self.save().await
    }

    pub async fn set_free_games_time(&self, hour: u32, minute: u32) -> Result<(), std::io::Error> {
        {
            let mut config = self.config.write().await;
            config.free_games_hour = hour.min(23);
            config.free_games_minute = minute.min(59);
        }
        self.save().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    /// `#[tokio::test]` is the async equivalent of `#[test]`.
    /// It creates a Tokio runtime for the test function.
    #[tokio::test]
    async fn test_config_default() {
        let config = BotConfig::default();
        assert_eq!(config.free_games_hour, 9);
        assert!(config.free_games_channel_id.is_none());
    }

    #[tokio::test]
    async fn test_config_save_load() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.json");

        let manager = ConfigManager::load(path.clone()).await;
        manager.set_free_games_channel(Some(12345)).await.unwrap();

        // Reload from disk to verify persistence
        let manager2 = ConfigManager::load(path).await;
        let config = manager2.get().await;
        assert_eq!(config.free_games_channel_id, Some(12345));
    }
}
