//! # General Commands
//!
//! ## Rust concepts covered:
//! - Procedural macros (`#[poise::command]`)
//! - Lifetime annotations (`'a`, `'static`)
//! - `&str` vs `String` (borrowed vs owned)
//! - Pattern matching with `match`
//! - `Option<T>` for nullable values
//! - `format!` macro for string interpolation

use crate::{Context, Error};

/// Check bot latency.
#[poise::command(slash_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    // ctx is borrowed (&) — we use it but don't take ownership.
    // "Pong!" is a &'static str — a string literal baked into the binary.
    ctx.say("Pong! 🏓").await?;
    Ok(())
}

/// Display help for all commands or a specific one.
#[poise::command(slash_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to display"] command: Option<String>,
) -> Result<(), Error> {
    let help_text = match command {
        // `match` is exhaustive — the compiler ensures all cases are handled.
        Some(cmd) => {
            // format! returns an owned String (heap-allocated)
            format!(
                "**Help for `/{}`**\n{}",
                cmd,
                get_command_help(&cmd) // &cmd borrows the String as &str
            )
        }
        None => {
            // Multiline string with `\` continuation (no newline inserted)
            String::from(
                "**LootCrab - Available commands**\n\n\
                **General:**\n\
                `/ping` - Check bot latency\n\
                `/help` - Show this help\n\
                `/uptime` - Time since bot started\n\n\
                **Developer:**\n\
                `/snippet` - Share a formatted code snippet\n\
                `/docs` - Search Rust documentation\n\n\
                **Timers:**\n\
                `/timer` - Set a reminder\n\
                `/pomodoro` - Start a Pomodoro session\n\n\
                **Gaming:**\n\
                `/freegames` - Show current free games",
            )
        }
    };

    ctx.say(help_text).await?;
    Ok(())
}

/// Returns a `&'static str` — a reference that lives for the entire program.
/// String literals are embedded in the binary, so they never get deallocated.
///
/// The parameter `&str` is a borrowed string slice — we read it without
/// taking ownership, so the caller keeps their String/&str.
fn get_command_help(command: &str) -> &'static str {
    match command {
        "ping" => "Sends a pong to check the bot's latency.",
        "help" => "Shows general help or help for a specific command.",
        "uptime" => "Shows how long the bot has been running.",
        "snippet" => "Shares a code snippet with syntax highlighting.\nUsage: `/snippet language code`",
        "docs" => "Searches the official Rust documentation.",
        "timer" => "Sets a reminder after a given duration.\nUsage: `/timer 5m Reminder message`",
        "pomodoro" => "Starts a Pomodoro session (25min work, 5min break).",
        "freegames" => "Displays currently free games from Epic Games and Steam.",
        // `_` is a catch-all pattern — matches anything not listed above
        _ => "Unknown command. Use `/help` to see all commands.",
    }
}

/// Show how long the bot has been running.
#[poise::command(slash_command)]
pub async fn uptime(ctx: Context<'_>) -> Result<(), Error> {
    // ctx.data() returns a reference to our shared Data struct
    let duration = ctx.data().start_time.elapsed();

    // Integer division and modulo for time formatting
    let hours = duration.as_secs() / 3600;
    let minutes = (duration.as_secs() % 3600) / 60;
    let seconds = duration.as_secs() % 60;

    let message = format!(
        "Bot online for **{}h {:02}m {:02}s**",
        hours, minutes, seconds
    );

    ctx.say(message).await?;
    Ok(())
}

/// Unit tests live in the same file, inside a `#[cfg(test)]` module.
/// `#[cfg(test)]` means this code is only compiled when running `cargo test`.
/// `use super::*` imports everything from the parent module.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_command_help_known_command() {
        let help = get_command_help("ping");
        assert!(!help.is_empty());
        assert!(help.contains("pong") || help.contains("latency"));
    }

    #[test]
    fn test_get_command_help_unknown_command() {
        let help = get_command_help("nonexistent_command");
        assert!(help.contains("Unknown"));
    }
}
