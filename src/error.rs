//! # Gestion des erreurs personnalisées
//!
//! ## Concepts Rust illustrés :
//! - Custom error types avec `thiserror`
//! - Le trait `std::error::Error`
//! - Conversion automatique avec `From`

use thiserror::Error;

/// Erreurs personnalisées du bot.
///
/// ## Concept Rust : thiserror
/// La crate `thiserror` génère l'implémentation de `std::error::Error`
/// et `Display` automatiquement via des macros.
///
/// ## Concept : #[error("...")] attribut
/// Définit le message d'erreur affiché pour chaque variant.
#[derive(Error, Debug)]
pub enum BotError {
    /// Erreur de configuration (variable manquante, etc.)
    #[error("Erreur de configuration: {0}")]
    Config(String),

    /// Erreur liée à l'API Discord
    #[error("Erreur Discord: {0}")]
    Discord(#[from] poise::serenity_prelude::Error),

    /// Erreur lors d'une requête HTTP externe
    #[error("Erreur HTTP: {0}")]
    Http(#[from] reqwest::Error),

    /// Erreur de parsing (durée, nombre, etc.)
    #[error("Erreur de parsing: {0}")]
    Parse(String),

    /// Erreur de permission (commande réservée aux admins, etc.)
    #[error("Permission refusée: {0}")]
    Permission(String),

    /// Erreur de rate limiting
    #[error("Trop de requêtes, réessaie dans {0} secondes")]
    RateLimit(u64),

    /// Erreur interne/inattendue
    #[error("Erreur interne: {0}")]
    Internal(String),
}

/// ## Concept Rust : impl From<T>
/// Permet la conversion automatique d'un type vers un autre.
/// Utilisé avec `?` pour convertir les erreurs automatiquement.
impl From<std::env::VarError> for BotError {
    fn from(err: std::env::VarError) -> Self {
        BotError::Config(err.to_string())
    }
}

/// Trait pour ajouter du contexte aux erreurs.
///
/// ## Concept Rust : Extension traits
/// On peut ajouter des méthodes à des types existants
/// en définissant un nouveau trait et en l'implémentant.
pub trait ResultExt<T> {
    /// Ajoute du contexte à une erreur.
    fn context(self, msg: &str) -> Result<T, BotError>;
}

impl<T, E: std::error::Error> ResultExt<T> for Result<T, E> {
    fn context(self, msg: &str) -> Result<T, BotError> {
        self.map_err(|e| BotError::Internal(format!("{}: {}", msg, e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = BotError::Config("DISCORD_TOKEN manquant".to_string());
        assert_eq!(
            err.to_string(),
            "Erreur de configuration: DISCORD_TOKEN manquant"
        );
    }

    #[test]
    fn test_rate_limit_error() {
        let err = BotError::RateLimit(30);
        assert!(err.to_string().contains("30"));
    }
}
