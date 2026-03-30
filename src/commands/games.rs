//! # Free Games Commands
//!
//! ## Rust concepts covered:
//! - Discord embeds with the builder pattern
//! - Iterators: `.iter()`, `.skip()`, `.filter()`, `.collect()`
//! - Closures as arguments to iterator methods
//! - `Colour::new(0x...)` — hexadecimal literals
//! - Helper functions returning owned vs borrowed types

use crate::services::free_games::{fetch_all_free_games, FreeGame, Store};
use crate::{Context, Error};
use poise::serenity_prelude::{self as serenity, ChannelType, Colour, CreateEmbed};

/// Display current free games (Epic Games + Steam).
#[poise::command(slash_command)]
pub async fn freegames(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let games = fetch_all_free_games().await;

    if games.is_empty() {
        ctx.say("📭 No free games found right now.").await?;
        return Ok(());
    }

    // Build embed for the first game (with image)
    let first = &games[0]; // &games[0] borrows the first element
    let mut embed = build_game_embed(first);

    // .iter() borrows the collection, .skip(1) creates a new iterator
    // starting from the second element
    for game in games.iter().skip(1) {
        let time_left = format_time_left(game);
        let value = format!(
            "{} **100% Discount** {} {}{}\n[Claim]({})",
            game.store.emoji(),
            store_separator(),
            game.store,
            if time_left.is_empty() { String::new() } else { format!(" {} {}", date_separator(), time_left) },
            game.url
        );
        // Builder pattern: each method returns the modified builder
        embed = embed.field(&game.title, value, false);
    }

    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}

/// Configure automatic free games notifications.
#[poise::command(
    slash_command,
    required_permissions = "MANAGE_CHANNELS",
    rename = "freegames-setup"
)]
pub async fn freegames_setup(
    ctx: Context<'_>,
    #[description = "Channel for notifications (leave empty to disable)"]
    channel: Option<serenity::Channel>,
    #[description = "Notification hour (0-23, default: 9)"]
    hour: Option<u32>,
    #[description = "Minutes (0-59, default: 0)"]
    minute: Option<u32>,
) -> Result<(), Error> {
    let config = &ctx.data().config;

    match channel {
        Some(ch) => {
            let channel_id = ch.id();

            if let Some(guild_channel) = ch.guild() {
                if guild_channel.kind != ChannelType::Text {
                    ctx.say("❌ This is not a text channel.").await?;
                    return Ok(());
                }
            }

            config.set_free_games_channel(Some(channel_id.get())).await
                .map_err(|e| format!("Save error: {}", e))?;

            // .min() clamps the value to the maximum
            let h = hour.unwrap_or(9).min(23);
            let m = minute.unwrap_or(0).min(59);
            config.set_free_games_time(h, m).await
                .map_err(|e| format!("Save error: {}", e))?;

            ctx.say(format!(
                "✅ **Free games notifications configured!**\n\n\
                📍 Channel: <#{}>\n\
                ⏰ Time: **{:02}:{:02}** daily\n\n\
                _You'll receive notifications with free Epic Games and Steam games._",
                channel_id, h, m
            )).await?;
        }
        None => {
            config.set_free_games_channel(None).await
                .map_err(|e| format!("Save error: {}", e))?;

            ctx.say("🔕 **Free games notifications disabled.**\n\n\
                _Use `/freegames-setup #channel` to re-enable._"
            ).await?;
        }
    }

    Ok(())
}

#[poise::command(
    slash_command,
    rename = "freegames-status"
)]
pub async fn freegames_status(ctx: Context<'_>) -> Result<(), Error> {
    // .get() acquires a read lock internally and returns a clone
    let config = ctx.data().config.get().await;

    let message = match config.free_games_channel_id {
        Some(channel_id) => {
            format!(
                "📊 **Free games notification config**\n\n\
                📍 Channel: <#{}>\n\
                ⏰ Time: **{:02}:{:02}** daily\n\
                ✅ Status: Enabled\n\n\
                _Use `/freegames-setup` to change the configuration._",
                channel_id,
                config.free_games_hour,
                config.free_games_minute
            )
        }
        None => {
            "📊 **Free games notification config**\n\n\
            ❌ Status: Disabled\n\n\
            _Use `/freegames-setup #channel` to enable notifications._".to_string()
        }
    };

    ctx.say(message).await?;
    Ok(())
}

/// Hexadecimal literals (0x...) for Discord embed colours.
fn store_colour(store: Store) -> Colour {
    match store {
        Store::EpicGames => Colour::new(0x2F2F2F),
        Store::Steam => Colour::new(0x1B2838),
    }
}

fn store_separator() -> &'static str {
    "·"
}

fn date_separator() -> &'static str {
    "📅"
}

fn format_time_left(game: &FreeGame) -> String {
    match game.end_date {
        Some(end) => {
            let remaining = end.signed_duration_since(chrono::Utc::now());
            let days = remaining.num_days();
            let hours = remaining.num_hours() % 24;

            if days > 0 {
                format!("{:02}/{:02}/{}", end.format("%d"), end.format("%m"), end.format("%Y"))
            } else if hours > 0 {
                format!("{}h left", hours)
            } else {
                "Last hours!".to_string()
            }
        }
        None => String::new(),
    }
}

/// Builder pattern: `CreateEmbed::new()` returns an embed builder.
/// Each method (`.title()`, `.url()`, etc.) consumes and returns the builder,
/// allowing method chaining.
fn build_game_embed(game: &FreeGame) -> CreateEmbed {
    let time_left = format_time_left(game);

    let description = format!(
        "{} **100% Discount** {} {} {}{}\n\n[Claim the game]({})",
        game.store.emoji(),
        store_separator(),
        game.store,
        if time_left.is_empty() { String::new() } else { format!("{} {}", date_separator(), time_left) },
        if let Some(price) = &game.original_price { format!("\n~~{}~~ → **Free**", price) } else { String::new() },
        game.url
    );

    let mut embed = CreateEmbed::new()
        .title(&game.title)
        .url(&game.url)
        .description(description)
        .colour(store_colour(game.store))
        .footer(serenity::CreateEmbedFooter::new("LootCrab"));

    if let Some(image_url) = &game.image_url {
        embed = embed.image(image_url);
    }

    embed
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    #[test]
    fn test_build_game_embed() {
        let game = FreeGame {
            title: "Test Game".to_string(),
            store: Store::EpicGames,
            url: "https://example.com".to_string(),
            original_price: Some("29.99€".to_string()),
            end_date: Some(Utc::now() + Duration::days(3)),
            image_url: None,
        };

        let _embed = build_game_embed(&game);
    }

    #[test]
    fn test_format_time_left() {
        let game = FreeGame {
            title: "Test".to_string(),
            store: Store::Steam,
            url: "https://example.com".to_string(),
            original_price: None,
            end_date: Some(Utc::now() + Duration::hours(5)),
            image_url: None,
        };
        let time = format_time_left(&game);
        assert!(time.contains("h left"));

        let game_no_date = FreeGame {
            title: "Test".to_string(),
            store: Store::Steam,
            url: "https://example.com".to_string(),
            original_price: None,
            end_date: None,
            image_url: None,
        };
        assert!(format_time_left(&game_no_date).is_empty());
    }
}
