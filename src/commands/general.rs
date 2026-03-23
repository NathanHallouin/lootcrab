//! # Commandes générales
//!
//! ## Concepts Rust illustrés :
//! - Macros procédurales (`#[poise::command]`)
//! - Lifetime annotations (`'a`)
//! - Async functions

use crate::{Context, Error};

/// Commande ping - Vérifie que le bot répond.
///
/// ## Concept Rust : Attributs et Macros
/// `#[poise::command(...)]` est une macro procédurale qui génère
/// du code pour transformer cette fonction en commande Discord.
#[poise::command(slash_command, prefix_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    // ## Concept : Ownership et borrowing
    // `ctx` est emprunté (borrowed) par la fonction
    // On utilise `&str` (string slice) plutôt que `String` (owned)
    ctx.say("Pong! 🏓").await?;
    Ok(())
}

/// Commande help - Affiche l'aide des commandes.
///
/// ## Concept Rust : Documentation
/// Les commentaires `///` génèrent de la documentation
/// qui peut être consultée avec `cargo doc`.
#[poise::command(slash_command, prefix_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Commande spécifique à afficher"] command: Option<String>,
) -> Result<(), Error> {
    // ## Concept : Option<T>
    // Option représente une valeur optionnelle : Some(valeur) ou None
    // C'est l'alternative Rust aux null/nil d'autres langages

    let help_text = match command {
        // ## Concept : Pattern matching
        // `match` permet de gérer tous les cas possibles
        Some(cmd) => {
            // ## Concept : format! macro
            // Similaire à printf, mais retourne une String
            format!(
                "**Aide pour `/{}`**\n{}",
                cmd,
                get_command_help(&cmd)
            )
        }
        None => {
            // ## Concept : String multiligne avec indentation préservée
            String::from(
                "**DevGamer Bot - Commandes disponibles**\n\n\
                **Général:**\n\
                `/ping` - Vérifie que le bot répond\n\
                `/help` - Affiche cette aide\n\
                `/uptime` - Temps depuis le démarrage du bot\n\n\
                **Développeur:**\n\
                `/snippet` - Partage un snippet de code formaté\n\
                `/docs` - Recherche dans la doc Rust\n\n\
                **Timers:**\n\
                `/timer` - Définit un rappel\n\
                `/pomodoro` - Lance une session Pomodoro",
            )
        }
    };

    ctx.say(help_text).await?;
    Ok(())
}

/// Fonction helper pour obtenir l'aide d'une commande spécifique.
///
/// ## Concept Rust : &str vs String
/// - `&str` : Référence immuable vers une chaîne (borrowed)
/// - `String` : Chaîne possédée (owned), allouée sur le heap
fn get_command_help(command: &str) -> &'static str {
    // ## Concept : 'static lifetime
    // Les string literals ont une durée de vie 'static
    // car ils sont intégrés dans le binaire
    match command {
        "ping" => "Envoie un pong pour vérifier la latence du bot.",
        "help" => "Affiche l'aide générale ou pour une commande spécifique.",
        "uptime" => "Affiche depuis combien de temps le bot est en ligne.",
        "snippet" => "Partage un snippet de code avec coloration syntaxique.\nUsage: `/snippet langage code`",
        "docs" => "Recherche dans la documentation Rust officielle.",
        "timer" => "Définit un rappel après un certain temps.\nUsage: `/timer 5m Message de rappel`",
        "pomodoro" => "Lance une session Pomodoro (25min travail, 5min pause).",
        _ => "Commande non reconnue. Utilisez `/help` pour voir toutes les commandes.",
    }
}

/// Commande uptime - Affiche le temps depuis le démarrage.
///
/// ## Concept Rust : Accès aux données partagées
/// On accède à `Data` via `ctx.data()` qui retourne une référence.
#[poise::command(slash_command, prefix_command)]
pub async fn uptime(ctx: Context<'_>) -> Result<(), Error> {
    // ## Concept : Instant et Duration
    // std::time::Instant représente un point dans le temps
    // elapsed() retourne la Duration depuis cet instant
    let duration = ctx.data().start_time.elapsed();

    // ## Concept : as - casting de types
    let hours = duration.as_secs() / 3600;
    let minutes = (duration.as_secs() % 3600) / 60;
    let seconds = duration.as_secs() % 60;

    let message = format!(
        "Bot en ligne depuis **{}h {:02}m {:02}s**",
        hours, minutes, seconds
    );

    ctx.say(message).await?;
    Ok(())
}

// ## Concept Rust : Tests unitaires
// Les tests sont dans le même fichier, dans un module `tests`
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_command_help_known_command() {
        let help = get_command_help("ping");
        assert!(!help.is_empty());
        assert!(help.contains("pong") || help.contains("latence"));
    }

    #[test]
    fn test_get_command_help_unknown_command() {
        let help = get_command_help("commande_inexistante");
        assert!(help.contains("non reconnue"));
    }
}
