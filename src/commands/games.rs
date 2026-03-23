//! # Commandes liées aux jeux
//!
//! ## Concepts Rust illustrés :
//! - Commandes avec sous-commandes
//! - Appel de services async
//! - Formatage de messages riches
//! - Permissions Discord

use crate::services::free_games::{fetch_all_free_games, FreeGame, Store};
use crate::{Context, Error};
use poise::serenity_prelude::{self as serenity, ChannelType};

/// Commande pour afficher les jeux gratuits actuels.
///
/// ## Concept Rust : Documentation des commandes
/// La doc string devient la description de la commande Discord.
#[poise::command(slash_command, prefix_command)]
pub async fn freegames(ctx: Context<'_>) -> Result<(), Error> {
    // Indiquer qu'on travaille (typing indicator)
    // ## Concept : defer pour les opérations longues
    ctx.defer().await?;

    let games = fetch_all_free_games().await;

    if games.is_empty() {
        ctx.say("📭 Aucun jeu gratuit trouvé actuellement.").await?;
        return Ok(());
    }

    // Construire le message
    let message = format_games_embed(&games);
    ctx.say(message).await?;

    Ok(())
}

/// Configure les notifications automatiques de jeux gratuits.
///
/// ## Concept Rust : Permissions Discord
/// `required_permissions` restreint la commande aux utilisateurs avec ces permissions.
#[poise::command(
    slash_command,
    prefix_command,
    required_permissions = "MANAGE_CHANNELS",
    rename = "freegames-setup"
)]
pub async fn freegames_setup(
    ctx: Context<'_>,
    #[description = "Channel pour les notifications (laisser vide pour désactiver)"]
    channel: Option<serenity::Channel>,
    #[description = "Heure de notification (0-23, défaut: 9)"]
    hour: Option<u32>,
    #[description = "Minutes (0-59, défaut: 0)"]
    minute: Option<u32>,
) -> Result<(), Error> {
    let config = &ctx.data().config;

    // ## Concept : Match avec Option
    match channel {
        Some(ch) => {
            // Vérifier que c'est un channel texte
            let channel_id = ch.id();

            // Récupérer le guild channel pour vérifier le type
            if let Some(guild_channel) = ch.guild() {
                if guild_channel.kind != ChannelType::Text {
                    ctx.say("❌ Ce channel n'est pas un channel texte.").await?;
                    return Ok(());
                }
            }

            // Configurer le channel
            config.set_free_games_channel(Some(channel_id.get())).await
                .map_err(|e| format!("Erreur de sauvegarde: {}", e))?;

            // Configurer l'heure si spécifiée
            let h = hour.unwrap_or(9).min(23);
            let m = minute.unwrap_or(0).min(59);
            config.set_free_games_time(h, m).await
                .map_err(|e| format!("Erreur de sauvegarde: {}", e))?;

            ctx.say(format!(
                "✅ **Notifications de jeux gratuits configurées !**\n\n\
                📍 Channel: <#{}>\n\
                ⏰ Heure: **{:02}:{:02}** tous les jours\n\n\
                _Tu recevras une notification avec les jeux gratuits Epic Games et Steam._",
                channel_id, h, m
            )).await?;
        }
        None => {
            // Désactiver les notifications
            config.set_free_games_channel(None).await
                .map_err(|e| format!("Erreur de sauvegarde: {}", e))?;

            ctx.say("🔕 **Notifications de jeux gratuits désactivées.**\n\n\
                _Utilise `/freegames-setup #channel` pour les réactiver._"
            ).await?;
        }
    }

    Ok(())
}

/// Affiche la configuration actuelle des notifications.
#[poise::command(
    slash_command,
    prefix_command,
    rename = "freegames-status"
)]
pub async fn freegames_status(ctx: Context<'_>) -> Result<(), Error> {
    let config = ctx.data().config.get().await;

    let message = match config.free_games_channel_id {
        Some(channel_id) => {
            format!(
                "📊 **Configuration des notifications de jeux gratuits**\n\n\
                📍 Channel: <#{}>\n\
                ⏰ Heure: **{:02}:{:02}** tous les jours\n\
                ✅ Status: Activé\n\n\
                _Utilise `/freegames-setup` pour modifier la configuration._",
                channel_id,
                config.free_games_hour,
                config.free_games_minute
            )
        }
        None => {
            "📊 **Configuration des notifications de jeux gratuits**\n\n\
            ❌ Status: Désactivé\n\n\
            _Utilise `/freegames-setup #channel` pour activer les notifications._".to_string()
        }
    };

    ctx.say(message).await?;
    Ok(())
}

/// Formate les jeux en message Discord avec sections.
fn format_games_embed(games: &[FreeGame]) -> String {
    let mut msg = String::from("# 🎮 Jeux Gratuits Actuels\n\n");

    let epic: Vec<_> = games.iter().filter(|g| g.store == Store::EpicGames).collect();
    let steam: Vec<_> = games.iter().filter(|g| g.store == Store::Steam).collect();

    if !epic.is_empty() {
        msg.push_str("## 🎁 Epic Games Store\n");
        msg.push_str(&format_table(&epic));
        msg.push('\n');
    }

    if !steam.is_empty() {
        msg.push_str("## 🎮 Steam\n");
        msg.push_str(&format_table(&steam));
        msg.push('\n');
    }

    msg.push_str("---\n💡 *Utilise `/freegames` pour rafraîchir la liste*");

    msg
}

/// Formate une liste de jeux en pseudo-tableau.
fn format_table(games: &[&FreeGame]) -> String {
    games
        .iter()
        .map(|game| {
            let mut line = format!("• **[{}]({})**", game.title, game.url);

            if let Some(end) = game.end_date {
                let remaining = end.signed_duration_since(chrono::Utc::now());
                let days = remaining.num_days();
                let hours = remaining.num_hours() % 24;

                let time_str = if days > 0 {
                    format!("{}j {}h", days, hours)
                } else if hours > 0 {
                    format!("{}h", hours)
                } else {
                    "⚠️ Dernières heures!".to_string()
                };

                line.push_str(&format!(" — ⏰ {}", time_str));
            }

            if let Some(price) = &game.original_price {
                line.push_str(&format!(" ~~{}~~", price));
            }

            line.push('\n');
            line
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    #[test]
    fn test_format_table() {
        let game = FreeGame {
            title: "Test Game".to_string(),
            store: Store::EpicGames,
            url: "https://example.com".to_string(),
            original_price: Some("29.99€".to_string()),
            end_date: Some(Utc::now() + Duration::days(3)),
            image_url: None,
        };

        let table = format_table(&[&game]);
        assert!(table.contains("Test Game"));
        assert!(table.contains("29.99€"));
        assert!(table.contains("j") || table.contains("h"));
    }
}
