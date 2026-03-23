//! # Scheduler pour les tâches planifiées
//!
//! ## Concepts Rust illustrés :
//! - Tokio tasks longue durée
//! - Calcul de temps jusqu'à la prochaine exécution
//! - Lecture dynamique de configuration

use chrono::{Local, NaiveTime};
use poise::serenity_prelude::{ChannelId, Http};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

use super::config::ConfigManager;
use super::free_games::{fetch_all_free_games, FreeGame, Store};

/// Scheduler de notifications de jeux gratuits.
///
/// ## Concept Rust : Configuration dynamique
/// Le scheduler lit la config à chaque cycle, permettant
/// de changer le channel ou l'heure sans redémarrer le bot.
#[derive(Clone)]
pub struct FreeGamesScheduler {
    /// Client HTTP Discord pour envoyer les messages
    http: Arc<Http>,
    /// Gestionnaire de configuration
    config: ConfigManager,
}

impl FreeGamesScheduler {
    /// Crée un nouveau scheduler.
    pub fn new(http: Arc<Http>, config: ConfigManager) -> Self {
        Self { http, config }
    }

    /// Calcule le temps restant jusqu'à la prochaine exécution.
    fn duration_until_next_run(hour: u32, minute: u32) -> Duration {
        let now = Local::now();
        let target_time = NaiveTime::from_hms_opt(hour, minute, 0)
            .unwrap_or_else(|| NaiveTime::from_hms_opt(9, 0, 0).unwrap());

        let today_run = now.date_naive().and_time(target_time);

        let next_run = if now.time() < target_time {
            today_run
        } else {
            today_run + chrono::Duration::days(1)
        };

        let duration_chrono = next_run.signed_duration_since(now.naive_local());
        Duration::from_secs(duration_chrono.num_seconds().max(0) as u64)
    }

    /// Démarre le scheduler en boucle infinie.
    ///
    /// ## Concept Rust : Configuration dynamique
    /// On relit la config à chaque itération, permettant des changements à chaud.
    pub async fn run(self) {
        tracing::info!("Scheduler de jeux gratuits démarré");

        loop {
            // Lire la configuration actuelle
            let config = self.config.get().await;

            // Vérifier si un channel est configuré
            let channel_id = match config.free_games_channel_id {
                Some(id) => ChannelId::new(id),
                None => {
                    // Pas de channel configuré, attendre et réessayer
                    tracing::debug!("Aucun channel configuré pour les jeux gratuits");
                    sleep(Duration::from_secs(60)).await;
                    continue;
                }
            };

            // Calculer le temps jusqu'à la prochaine notification
            let wait_duration = Self::duration_until_next_run(
                config.free_games_hour,
                config.free_games_minute,
            );

            tracing::info!(
                "Prochaine notification de jeux gratuits dans {}h {:02}m (channel: {})",
                wait_duration.as_secs() / 3600,
                (wait_duration.as_secs() % 3600) / 60,
                channel_id
            );

            // Attendre jusqu'à l'heure de notification
            // On vérifie toutes les 5 minutes si la config a changé
            let check_interval = Duration::from_secs(300); // 5 minutes
            let mut remaining = wait_duration;

            while remaining > Duration::ZERO {
                let sleep_time = remaining.min(check_interval);
                sleep(sleep_time).await;
                remaining = remaining.saturating_sub(sleep_time);

                // Vérifier si la config a changé
                let new_config = self.config.get().await;
                if new_config.free_games_channel_id != config.free_games_channel_id
                    || new_config.free_games_hour != config.free_games_hour
                    || new_config.free_games_minute != config.free_games_minute
                {
                    tracing::info!("Configuration modifiée, recalcul du prochain envoi");
                    break;
                }
            }

            // Si on a attendu tout le temps (pas de changement de config), envoyer
            if remaining == Duration::ZERO {
                if let Err(e) = self.send_free_games_notification(channel_id).await {
                    tracing::error!("Erreur lors de l'envoi de la notification: {}", e);
                }

                // Attendre 1 minute pour éviter les doubles envois
                sleep(Duration::from_secs(60)).await;
            }
        }
    }

    /// Envoie la notification des jeux gratuits.
    async fn send_free_games_notification(
        &self,
        channel_id: ChannelId,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        tracing::info!("Récupération des jeux gratuits...");

        let games = fetch_all_free_games().await;

        if games.is_empty() {
            channel_id
                .say(&self.http, "📭 Aucun jeu gratuit trouvé aujourd'hui.")
                .await?;
            return Ok(());
        }

        let message = format_games_message(&games);
        channel_id.say(&self.http, &message).await?;

        tracing::info!("Notification envoyée: {} jeux gratuits", games.len());

        Ok(())
    }
}

/// Formate les jeux en un message Discord lisible.
pub fn format_games_message(games: &[FreeGame]) -> String {
    let mut message = String::from("# 🎮 Jeux Gratuits du Jour\n\n");

    let epic_games: Vec<_> = games.iter().filter(|g| g.store == Store::EpicGames).collect();
    let steam_games: Vec<_> = games.iter().filter(|g| g.store == Store::Steam).collect();

    if !epic_games.is_empty() {
        message.push_str("## 🎁 Epic Games Store\n");
        for game in &epic_games {
            message.push_str(&format_game_line(game));
        }
        message.push('\n');
    }

    if !steam_games.is_empty() {
        message.push_str("## 🎮 Steam\n");
        for game in &steam_games {
            message.push_str(&format_game_line(game));
        }
        message.push('\n');
    }

    message.push_str("---\n*Récupère-les avant qu'ils ne soient plus gratuits !*");

    message
}

/// Formate une ligne pour un jeu.
fn format_game_line(game: &FreeGame) -> String {
    let mut line = format!("• **[{}]({})**", game.title, game.url);

    if let Some(end_date) = game.end_date {
        let remaining = end_date.signed_duration_since(chrono::Utc::now());
        let days = remaining.num_days();
        let hours = remaining.num_hours() % 24;

        if days > 0 {
            line.push_str(&format!(" — ⏰ {}j {}h", days, hours));
        } else if hours > 0 {
            line.push_str(&format!(" — ⏰ {}h", hours));
        } else {
            line.push_str(" — ⏰ **Dernières heures !**");
        }
    }

    if let Some(price) = &game.original_price {
        line.push_str(&format!(" ~~{}~~", price));
    }

    line.push('\n');
    line
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_game_line() {
        let game = FreeGame {
            title: "Test Game".to_string(),
            store: Store::EpicGames,
            url: "https://example.com".to_string(),
            original_price: Some("19.99€".to_string()),
            end_date: None,
            image_url: None,
        };

        let line = format_game_line(&game);
        assert!(line.contains("Test Game"));
        assert!(line.contains("19.99€"));
    }

    #[test]
    fn test_duration_until_next_run() {
        // Ce test vérifie que la durée calculée est positive et raisonnable
        let duration = FreeGamesScheduler::duration_until_next_run(9, 0);
        assert!(duration.as_secs() <= 24 * 60 * 60); // Max 24h
    }
}
