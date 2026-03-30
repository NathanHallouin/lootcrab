//! # Free Games Scheduler
//!
//! ## Rust concepts covered:
//! - Long-running async loops
//! - `Duration` arithmetic with `.saturating_sub()` and `.min()`
//! - Dynamic configuration reloading without restart
//! - `Arc<Http>` for sharing the Discord client across tasks
//! - Struct methods with `self`, `&self`, and `Self`

use chrono::{Local, NaiveTime};
use poise::serenity_prelude::{ChannelId, Colour, CreateEmbed, CreateEmbedFooter, CreateMessage, Http};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

use super::config::ConfigManager;
use super::free_games::{fetch_all_free_games, FreeGame, Store};

/// `Arc<Http>` lets multiple tasks share the same HTTP client.
/// `Arc` (Atomic Reference Counting) is the thread-safe version of `Rc`.
#[derive(Clone)]
pub struct FreeGamesScheduler {
    http: Arc<Http>,
    config: ConfigManager,
}

impl FreeGamesScheduler {
    /// `Self` is an alias for the implementing type (FreeGamesScheduler).
    /// This is an associated function (no `self` parameter) — called with `::`.
    pub fn new(http: Arc<Http>, config: ConfigManager) -> Self {
        Self { http, config }
    }

    /// Calculates time until the next scheduled run.
    /// Uses chrono for date/time arithmetic — Rust has no built-in date library.
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
        // .max(0) ensures we never get a negative duration
        Duration::from_secs(duration_chrono.num_seconds().max(0) as u64)
    }

    /// Takes `self` by value (ownership) — the scheduler is consumed and runs forever.
    /// This is intentional: the caller gives up the scheduler to the background loop.
    pub async fn run(self) {
        tracing::info!("Free games scheduler started");

        loop {
            let config = self.config.get().await;

            let channel_id = match config.free_games_channel_id {
                Some(id) => ChannelId::new(id),
                None => {
                    tracing::debug!("No channel configured for free games");
                    sleep(Duration::from_secs(60)).await;
                    // `continue` skips to the next loop iteration
                    continue;
                }
            };

            let wait_duration = Self::duration_until_next_run(
                config.free_games_hour,
                config.free_games_minute,
            );

            tracing::info!(
                "Next free games notification in {}h {:02}m (channel: {})",
                wait_duration.as_secs() / 3600,
                (wait_duration.as_secs() % 3600) / 60,
                channel_id
            );

            // Poll every 5 minutes to check if config has changed.
            // `.min()` returns the smaller of two values.
            // `.saturating_sub()` subtracts without underflow (stops at zero).
            let check_interval = Duration::from_secs(300);
            let mut remaining = wait_duration;

            while remaining > Duration::ZERO {
                let sleep_time = remaining.min(check_interval);
                sleep(sleep_time).await;
                remaining = remaining.saturating_sub(sleep_time);

                let new_config = self.config.get().await;
                if new_config.free_games_channel_id != config.free_games_channel_id
                    || new_config.free_games_hour != config.free_games_hour
                    || new_config.free_games_minute != config.free_games_minute
                {
                    tracing::info!("Config changed, recalculating next run");
                    break;
                }
            }

            if remaining == Duration::ZERO {
                if let Err(e) = self.send_free_games_notification(channel_id).await {
                    tracing::error!("Failed to send notification: {}", e);
                }

                sleep(Duration::from_secs(60)).await;
            }
        }
    }

    /// `&self` borrows the scheduler — we can call this method multiple times.
    async fn send_free_games_notification(
        &self,
        channel_id: ChannelId,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        tracing::info!("Fetching free games...");

        let games = fetch_all_free_games().await;

        if games.is_empty() {
            channel_id
                .say(&self.http, "📭 No free games found today.")
                .await?;
            return Ok(());
        }

        let embeds = build_notification_embeds(&games);
        channel_id.send_message(&self.http, CreateMessage::new().embeds(embeds)).await?;

        tracing::info!("Notification sent: {} free games", games.len());

        Ok(())
    }
}

/// Builds Discord embeds for each game.
/// `.take(10)` limits to 10 — Discord's maximum embeds per message.
/// `.map()` transforms each element, `.collect()` gathers into a Vec.
fn build_notification_embeds(games: &[FreeGame]) -> Vec<CreateEmbed> {
    games.iter().take(10).map(|game| {
        let time_left = format_time_left(game);
        let colour = match game.store {
            Store::EpicGames => Colour::new(0x2F2F2F),
            Store::Steam => Colour::new(0x1B2838),
        };

        let description = format!(
            "{} **100% Discount** · {}{}\n\n[Claim the game]({})",
            game.store.emoji(),
            game.store,
            if time_left.is_empty() { String::new() } else { format!(" 📅 {}", time_left) },
            game.url
        );

        let mut embed = CreateEmbed::new()
            .title(&game.title)
            .url(&game.url)
            .description(description)
            .colour(colour)
            .footer(CreateEmbedFooter::new("LootCrab"));

        if let Some(image_url) = &game.image_url {
            embed = embed.image(image_url);
        }

        embed
    }).collect()
}

fn format_time_left(game: &FreeGame) -> String {
    match game.end_date {
        Some(end) => {
            let remaining = end.signed_duration_since(chrono::Utc::now());
            let days = remaining.num_days();
            let hours = remaining.num_hours() % 24;

            if days > 0 {
                format!("{}", end.format("%d/%m/%Y"))
            } else if hours > 0 {
                format!("{}h left", hours)
            } else {
                "Last hours!".to_string()
            }
        }
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_notification_embeds() {
        let game = FreeGame {
            title: "Test Game".to_string(),
            store: Store::EpicGames,
            url: "https://example.com".to_string(),
            original_price: Some("19.99€".to_string()),
            end_date: None,
            image_url: None,
        };

        let embeds = build_notification_embeds(&[game]);
        assert_eq!(embeds.len(), 1);
    }

    #[test]
    fn test_duration_until_next_run() {
        let duration = FreeGamesScheduler::duration_until_next_run(9, 0);
        assert!(duration.as_secs() <= 24 * 60 * 60);
    }
}
