//! # LootCrab - Entry Point
//!
//! ## Rust concepts covered:
//! - `async/await` with the Tokio runtime
//! - `Result<T, E>` for explicit error handling
//! - `?` operator for error propagation
//! - Module system and code organization
//! - Type aliases for complex types
//! - Closures and `move` semantics
//! - `Box::pin` for heap-allocated futures

use poise::serenity_prelude as serenity;
use std::path::PathBuf;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod commands;
mod services;

use services::config::ConfigManager;
use services::scheduler::FreeGamesScheduler;

/// Shared state accessible from all commands.
///
/// `ConfigManager` uses `Arc<RwLock>` internally, so cloning is cheap
/// (only increments a reference counter).
#[derive(Clone)]
pub struct Data {
    pub start_time: std::time::Instant,
    pub config: ConfigManager,
}

/// Type aliases simplify complex generic signatures.
///
/// `Box<dyn Error + Send + Sync>` is a trait object: any error type behind a pointer.
/// `Send + Sync` bounds are required for use across async task boundaries.
pub type Error = Box<dyn std::error::Error + Send + Sync>;

/// The `'a` lifetime ensures the context cannot outlive the data it borrows.
pub type Context<'a> = poise::Context<'a, Data, Error>;

/// `#[tokio::main]` is a procedural macro that sets up the async runtime.
/// It transforms `async fn main()` into a synchronous `fn main()` that
/// creates a Tokio runtime and blocks on the future.
#[tokio::main]
async fn main() -> Result<(), Error> {
    // .ok() discards the Result — it's fine if .env doesn't exist
    dotenvy::dotenv().ok();

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // .expect() panics with a message on Err — appropriate here since
    // the bot cannot function without a token (unrecoverable error)
    let token = std::env::var("DISCORD_TOKEN")
        .expect("Missing DISCORD_TOKEN in .env");

    let config_path = PathBuf::from("data/config.json");
    let config = ConfigManager::load(config_path).await;

    info!("Configuration loaded");

    // Gateway intents control which events the bot receives.
    // non_privileged() excludes MESSAGE_CONTENT (not needed for slash commands).
    let intents = serenity::GatewayIntents::non_privileged();

    // Clone before the `move` closure captures ownership
    let config_for_setup = config.clone();

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
            // Struct update syntax: fill remaining fields with Default
            ..Default::default()
        })
        // `move` transfers ownership of captured variables into the closure.
        // `Box::pin` is needed because async closures return unsized Futures.
        .setup(move |ctx, _ready, framework| {
            let http = ctx.http.clone();
            let config = config_for_setup;

            Box::pin(async move {
                // Register slash commands: guild-scoped (instant) in dev,
                // global (up to 1h delay) in production
                if let Ok(guild_id) = std::env::var("DEV_GUILD_ID") {
                    if let Ok(id) = guild_id.parse::<u64>() {
                        let guild = serenity::GuildId::new(id);
                        poise::builtins::register_in_guild(ctx, &framework.options().commands, guild).await?;
                        info!("Slash commands registered on dev guild ({})", id);
                    }
                } else {
                    poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                }
                info!("Bot started and ready!");

                // tokio::spawn launches an independent async task.
                // `async move` takes ownership of captured variables.
                let scheduler = FreeGamesScheduler::new(http, config.clone());
                tokio::spawn(async move {
                    scheduler.run().await;
                });

                info!("Free games scheduler started (dynamic config via /freegames-setup)");

                Ok(Data {
                    start_time: std::time::Instant::now(),
                    config,
                })
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    // Pattern matching on Result to handle success/failure explicitly
    match client {
        Ok(mut client) => {
            info!("Connecting to Discord...");
            client.start().await?;
        }
        Err(e) => {
            tracing::error!("Failed to create client: {}", e);
            // .into() converts serenity::Error into Box<dyn Error>
            return Err(e.into());
        }
    }

    Ok(())
}
