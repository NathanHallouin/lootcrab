//! # Modèles de données
//!
//! ## Concepts Rust illustrés :
//! - Structs avec derive macros
//! - Serde pour la sérialisation/désérialisation
//! - Builder pattern

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Représente un rappel/timer.
///
/// ## Concept Rust : Derive macros multiples
/// On peut dériver plusieurs traits en même temps.
/// Serde permet de convertir vers/depuis JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reminder {
    /// ID unique du rappel
    pub id: u64,
    /// ID de l'utilisateur Discord
    pub user_id: u64,
    /// ID du channel où envoyer le rappel
    pub channel_id: u64,
    /// Message du rappel
    pub message: String,
    /// Moment où le rappel doit être envoyé
    pub trigger_at: DateTime<Utc>,
    /// Moment de création
    pub created_at: DateTime<Utc>,
}

impl Reminder {
    /// Crée un nouveau rappel.
    ///
    /// ## Concept Rust : Associated functions
    /// `new` est une convention pour les constructeurs.
    /// `Self` est un alias pour le type actuel.
    pub fn new(
        id: u64,
        user_id: u64,
        channel_id: u64,
        message: String,
        trigger_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            user_id,
            channel_id,
            message,
            trigger_at,
            created_at: Utc::now(),
        }
    }

    /// Vérifie si le rappel doit être déclenché.
    pub fn should_trigger(&self) -> bool {
        Utc::now() >= self.trigger_at
    }
}

/// Configuration d'une session Pomodoro.
///
/// ## Concept Rust : Default trait
/// Permet de créer une valeur par défaut pour un type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PomodoroConfig {
    /// Durée de travail en minutes
    pub work_duration: u64,
    /// Durée de pause courte en minutes
    pub short_break: u64,
    /// Durée de pause longue en minutes
    pub long_break: u64,
    /// Nombre de sessions avant la pause longue
    pub sessions_before_long_break: u32,
}

/// ## Concept Rust : impl Default
/// Implémente le trait Default pour fournir des valeurs par défaut.
impl Default for PomodoroConfig {
    fn default() -> Self {
        Self {
            work_duration: 25,
            short_break: 5,
            long_break: 15,
            sessions_before_long_break: 4,
        }
    }
}

/// Snippet de code partagé.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSnippet {
    pub language: String,
    pub code: String,
    pub description: Option<String>,
    pub author_id: u64,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pomodoro_default() {
        let config = PomodoroConfig::default();
        assert_eq!(config.work_duration, 25);
        assert_eq!(config.short_break, 5);
    }

    #[test]
    fn test_reminder_should_trigger() {
        use chrono::Duration;

        let past_reminder = Reminder::new(
            1,
            12345,
            67890,
            "Test".to_string(),
            Utc::now() - Duration::hours(1),
        );
        assert!(past_reminder.should_trigger());

        let future_reminder = Reminder::new(
            2,
            12345,
            67890,
            "Test".to_string(),
            Utc::now() + Duration::hours(1),
        );
        assert!(!future_reminder.should_trigger());
    }
}
