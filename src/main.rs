//! # DevGamer Bot - Point d'entrée
//!
//! ## Concepts Rust illustrés ici :
//! - `async/await` : Programmation asynchrone avec Tokio
//! - `Result<T, E>` : Gestion des erreurs explicite
//! - `?` operator : Propagation élégante des erreurs
//! - Modules : Organisation du code en modules

use poise::serenity_prelude as serenity;
use std::path::PathBuf;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod commands;
mod error;
mod models;
mod services;
mod utils;

use services::config::ConfigManager;
use services::scheduler::FreeGamesScheduler;

/// Données partagées entre toutes les commandes du bot.
///
/// ## Concept Rust : Struct avec Clone
/// Une struct permet de regrouper des données liées.
/// ConfigManager utilise Arc<RwLock> en interne, donc Clone est peu coûteux.
#[derive(Clone)]
pub struct Data {
    /// Timestamp de démarrage du bot (pour la commande uptime)
    pub start_time: std::time::Instant,
    /// Gestionnaire de configuration persistante
    pub config: ConfigManager,
}

/// Alias de type pour simplifier les signatures de fonction.
///
/// ## Concept Rust : Type Alias
/// `type` crée un alias pour un type complexe, améliorant la lisibilité.
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

/// Point d'entrée asynchrone du programme.
///
/// ## Concepts Rust :
/// - `#[tokio::main]` : Macro qui configure le runtime async
/// - `async fn` : Fonction asynchrone
/// - `Result<(), Error>` : Retourne Ok(()) ou une erreur
#[tokio::main]
async fn main() -> Result<(), Error> {
    // Charger les variables d'environnement depuis .env
    dotenvy::dotenv().ok();

    // Configurer le système de logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Récupérer le token Discord depuis l'environnement
    let token = std::env::var("DISCORD_TOKEN")
        .expect("Variable DISCORD_TOKEN manquante dans .env");

    // Charger la configuration persistante
    // ## Concept : PathBuf pour les chemins de fichiers
    let config_path = PathBuf::from("data/config.json");
    let config = ConfigManager::load(config_path).await;

    info!("Configuration chargée");

    // Configurer les intents Discord (permissions)
    let intents = serenity::GatewayIntents::non_privileged()
        | serenity::GatewayIntents::MESSAGE_CONTENT;

    // Cloner config pour le move dans setup
    let config_for_setup = config.clone();

    // Construire le framework avec toutes les commandes
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::general::ping(),
                commands::general::help(),
                commands::general::uptime(),
                commands::dev::snippet(),
                commands::dev::docs(),
                commands::timer::timer(),
                commands::timer::pomodoro(),
                commands::games::freegames(),
                commands::games::freegames_setup(),
                commands::games::freegames_status(),
            ],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("!".into()),
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            let http = ctx.http.clone();
            let config = config_for_setup;

            Box::pin(async move {
                // Enregistrer les slash commands
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                info!("Bot démarré et prêt !");

                // Démarrer le scheduler de jeux gratuits
                // ## Concept : Le scheduler tourne en arrière-plan et lit la config dynamiquement
                let scheduler = FreeGamesScheduler::new(http, config.clone());
                tokio::spawn(async move {
                    scheduler.run().await;
                });

                info!("Scheduler de jeux gratuits démarré (config dynamique via /freegames-setup)");

                // Retourner les données partagées
                Ok(Data {
                    start_time: std::time::Instant::now(),
                    config,
                })
            })
        })
        .build();

    // Créer et démarrer le client Discord
    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    match client {
        Ok(mut client) => {
            info!("Connexion à Discord...");
            client.start().await?;
        }
        Err(e) => {
            tracing::error!("Erreur lors de la création du client: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}
