//! # Service de configuration persistante
//!
//! ## Concepts Rust illustrés :
//! - Lecture/écriture de fichiers async
//! - Serde pour JSON
//! - Arc<RwLock> pour partage thread-safe avec mutation
//! - Paths et gestion de fichiers

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Configuration persistante du bot.
///
/// ## Concept Rust : Serde avec Option
/// Les champs `Option<T>` sont désérialisés comme `null` ou absents en JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotConfig {
    /// Channel où envoyer les notifications de jeux gratuits
    pub free_games_channel_id: Option<u64>,

    /// Heure de notification (0-23)
    #[serde(default = "default_hour")]
    pub free_games_hour: u32,

    /// Minutes de notification (0-59)
    #[serde(default)]
    pub free_games_minute: u32,
}

/// Valeur par défaut pour l'heure (9h00).
fn default_hour() -> u32 {
    9
}

/// ## Concept Rust : impl Default manuellement
/// On implémente Default à la main pour contrôler les valeurs par défaut.
impl Default for BotConfig {
    fn default() -> Self {
        Self {
            free_games_channel_id: None,
            free_games_hour: 9,
            free_games_minute: 0,
        }
    }
}

/// Gestionnaire de configuration thread-safe.
///
/// ## Concept Rust : Arc<RwLock<T>>
/// - `Arc` : Permet de partager entre plusieurs tasks async
/// - `RwLock` : Permet plusieurs lecteurs OU un seul écrivain
///   (plus efficace que Mutex quand on lit souvent)
#[derive(Clone)]
pub struct ConfigManager {
    config: Arc<RwLock<BotConfig>>,
    path: PathBuf,
}

impl ConfigManager {
    /// Charge la configuration depuis un fichier, ou crée une config par défaut.
    ///
    /// ## Concept Rust : async file I/O avec tokio
    pub async fn load(path: PathBuf) -> Self {
        let config = if path.exists() {
            // ## Concept : tokio::fs pour I/O async
            match tokio::fs::read_to_string(&path).await {
                Ok(content) => {
                    // ## Concept : serde_json::from_str
                    serde_json::from_str(&content).unwrap_or_default()
                }
                Err(e) => {
                    tracing::warn!("Impossible de lire la config: {}", e);
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

    /// Sauvegarde la configuration dans le fichier.
    ///
    /// ## Concept Rust : RwLock::read() pour lecture
    pub async fn save(&self) -> Result<(), std::io::Error> {
        // ## Concept : .read().await pour obtenir un read lock
        let config = self.config.read().await;

        // ## Concept : serde_json::to_string_pretty pour JSON lisible
        let json = serde_json::to_string_pretty(&*config)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        // Créer le dossier parent si nécessaire
        if let Some(parent) = self.path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::write(&self.path, json).await
    }

    /// Obtient une copie de la configuration actuelle.
    pub async fn get(&self) -> BotConfig {
        self.config.read().await.clone()
    }

    /// Configure le channel pour les notifications de jeux gratuits.
    ///
    /// ## Concept Rust : RwLock::write() pour écriture exclusive
    pub async fn set_free_games_channel(&self, channel_id: Option<u64>) -> Result<(), std::io::Error> {
        {
            // ## Concept : scope pour libérer le lock avant save()
            let mut config = self.config.write().await;
            config.free_games_channel_id = channel_id;
        }
        self.save().await
    }

    /// Configure l'heure de notification.
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

        // Recharger
        let manager2 = ConfigManager::load(path).await;
        let config = manager2.get().await;
        assert_eq!(config.free_games_channel_id, Some(12345));
    }
}
