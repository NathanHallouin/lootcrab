//! # Commandes de timer
//!
//! ## Concepts Rust illustrés :
//! - Async/await en profondeur
//! - tokio::time pour les délais
//! - Parsing de durées

use crate::{Context, Error};
use std::time::Duration;
use tokio::time::sleep;

/// Parse une durée depuis une string (ex: "5m", "1h30m", "90s").
///
/// ## Concept Rust : Result<T, E>
/// Plutôt que de retourner null ou lever une exception,
/// Rust utilise Result pour les opérations qui peuvent échouer.
///
/// ## Concept : From/Into traits
/// On pourrait implémenter `FromStr` pour un type custom Duration,
/// mais ici on utilise une fonction simple.
fn parse_duration(input: &str) -> Result<Duration, String> {
    // ## Concept : chars() et iterators
    // Les strings Rust sont UTF-8, on itère sur les caractères
    let input = input.trim().to_lowercase();

    // ## Concept : mutable variables
    // Par défaut tout est immutable, `mut` permet la modification
    let mut total_seconds: u64 = 0;
    let mut current_num = String::new();

    for c in input.chars() {
        // ## Concept : char methods
        if c.is_ascii_digit() {
            current_num.push(c);
        } else {
            // ## Concept : parse() avec type inference
            // Le compilateur infère le type cible depuis le contexte
            let num: u64 = current_num.parse().unwrap_or(0);

            match c {
                'h' => total_seconds += num * 3600,
                'm' => total_seconds += num * 60,
                's' => total_seconds += num,
                _ => return Err(format!("Unité invalide: '{}'", c)),
            }
            current_num.clear();
        }
    }

    // ## Concept : early return avec garde
    if total_seconds == 0 {
        return Err("Durée invalide. Exemples: 5m, 1h30m, 90s".to_string());
    }

    Ok(Duration::from_secs(total_seconds))
}

/// Commande timer - Définit un rappel.
///
/// ## Concept Rust : Async programming
/// La fonction est `async` car elle attend (sleep) sans bloquer.
/// Pendant l'attente, le runtime peut exécuter d'autres tâches.
#[poise::command(slash_command, prefix_command)]
pub async fn timer(
    ctx: Context<'_>,
    #[description = "Durée (ex: 5m, 1h30m, 90s)"] duration_str: String,
    #[description = "Message de rappel"] message: String,
) -> Result<(), Error> {
    // ## Concept : match avec Result
    let duration = match parse_duration(&duration_str) {
        Ok(d) => d,
        Err(e) => {
            ctx.say(format!("❌ Erreur: {}", e)).await?;
            return Ok(());
        }
    };

    // Limiter à 24h pour éviter les abus
    // ## Concept : constantes et comparaison
    const MAX_DURATION: Duration = Duration::from_secs(24 * 60 * 60);
    if duration > MAX_DURATION {
        ctx.say("❌ Durée maximale: 24 heures").await?;
        return Ok(());
    }

    // Confirmer le timer
    let minutes = duration.as_secs() / 60;
    let seconds = duration.as_secs() % 60;

    ctx.say(format!(
        "⏰ Timer défini pour **{}m {:02}s**\nMessage: {}",
        minutes, seconds, message
    ))
    .await?;

    // ## Concept : tokio::spawn
    // Crée une nouvelle tâche asynchrone indépendante.
    // Le timer continue même après la réponse à l'utilisateur.

    // ## Concept : Clone
    // On clone les données nécessaires car elles seront
    // utilisées dans une tâche séparée après le return
    let channel_id = ctx.channel_id();
    let http = ctx.serenity_context().http.clone();
    let author_id = ctx.author().id;

    tokio::spawn(async move {
        // ## Concept : await
        // `await` suspend l'exécution jusqu'à ce que le Future soit prêt
        sleep(duration).await;

        // Envoyer le rappel
        let reminder = format!("🔔 <@{}> Rappel: **{}**", author_id, message);

        // ## Concept : if let pour ignorer certaines erreurs
        if let Err(e) = channel_id.say(&http, reminder).await {
            tracing::error!("Erreur envoi rappel: {}", e);
        }
    });

    Ok(())
}

/// Commande pomodoro - Lance une session Pomodoro.
///
/// ## Concept : Technique Pomodoro
/// 25 minutes de travail, 5 minutes de pause.
#[poise::command(slash_command, prefix_command)]
pub async fn pomodoro(
    ctx: Context<'_>,
    #[description = "Durée de travail en minutes (défaut: 25)"] work_minutes: Option<u64>,
    #[description = "Durée de pause en minutes (défaut: 5)"] break_minutes: Option<u64>,
) -> Result<(), Error> {
    // ## Concept : unwrap_or pour valeurs par défaut
    let work = work_minutes.unwrap_or(25);
    let pause = break_minutes.unwrap_or(5);

    // ## Concept : Validation
    if work == 0 || work > 120 {
        ctx.say("❌ La durée de travail doit être entre 1 et 120 minutes")
            .await?;
        return Ok(());
    }

    ctx.say(format!(
        "🍅 **Session Pomodoro démarrée!**\n\
        ⏱️ Travail: {} minutes\n\
        ☕ Pause: {} minutes\n\n\
        Concentre-toi! Je te préviendrai quand ce sera fini.",
        work, pause
    ))
    .await?;

    let channel_id = ctx.channel_id();
    let http = ctx.serenity_context().http.clone();
    let author_id = ctx.author().id;

    tokio::spawn(async move {
        // Phase de travail
        sleep(Duration::from_secs(work * 60)).await;

        let work_done_msg = format!(
            "🍅 <@{}> **Temps de travail terminé!**\n\
            Prends une pause de {} minutes. Tu l'as bien mérité! ☕",
            author_id, pause
        );

        if let Err(e) = channel_id.say(&http, &work_done_msg).await {
            tracing::error!("Erreur notification pomodoro: {}", e);
            return;
        }

        // Phase de pause
        sleep(Duration::from_secs(pause * 60)).await;

        let break_done_msg = format!(
            "🍅 <@{}> **Pause terminée!**\n\
            Prêt pour une nouvelle session? Utilise `/pomodoro` pour recommencer!",
            author_id
        );

        if let Err(e) = channel_id.say(&http, break_done_msg).await {
            tracing::error!("Erreur notification fin pause: {}", e);
        }
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration_minutes() {
        let duration = parse_duration("5m").unwrap();
        assert_eq!(duration, Duration::from_secs(300));
    }

    #[test]
    fn test_parse_duration_hours_minutes() {
        let duration = parse_duration("1h30m").unwrap();
        assert_eq!(duration, Duration::from_secs(5400));
    }

    #[test]
    fn test_parse_duration_seconds() {
        let duration = parse_duration("90s").unwrap();
        assert_eq!(duration, Duration::from_secs(90));
    }

    #[test]
    fn test_parse_duration_invalid() {
        let result = parse_duration("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_duration_zero() {
        let result = parse_duration("0m");
        assert!(result.is_err());
    }
}
