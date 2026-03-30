//! # Timer Commands
//!
//! ## Rust concepts covered:
//! - Iterators: `.chars()`, `.parse()`
//! - Mutability: `let mut` for variables that change
//! - `tokio::spawn` for background tasks
//! - `async move` closures and ownership transfer
//! - Early returns with `return Err(...)`
//! - `const` for compile-time constants

use crate::{Context, Error};
use std::time::Duration;
use tokio::time::sleep;

/// Parses a duration string like "5m", "1h30m", "90s".
///
/// Returns `Result<Duration, String>` — Rust's way of handling operations
/// that can fail. No exceptions, no null — the caller must handle both cases.
fn parse_duration(input: &str) -> Result<Duration, String> {
    // .trim() returns &str, .to_lowercase() returns a new String.
    // Shadowing: reusing `input` with a different type is idiomatic Rust.
    let input = input.trim().to_lowercase();

    // `mut` is required for variables that will be modified.
    // Everything is immutable by default in Rust.
    let mut total_seconds: u64 = 0;
    let mut current_num = String::new();

    // .chars() returns an iterator over Unicode characters.
    // Rust strings are UTF-8, so indexing by byte isn't safe — iterate instead.
    for c in input.chars() {
        if c.is_ascii_digit() {
            current_num.push(c);
        } else {
            // .parse() uses type inference — the compiler knows we want u64.
            // .unwrap_or(0) provides a fallback value instead of panicking.
            let num: u64 = current_num.parse().unwrap_or(0);

            match c {
                'h' => total_seconds += num * 3600,
                'm' => total_seconds += num * 60,
                's' => total_seconds += num,
                _ => return Err(format!("Invalid unit: '{}'", c)),
            }
            current_num.clear();
        }
    }

    // Early return — exits the function immediately with an Err
    if total_seconds == 0 {
        return Err("Invalid duration. Examples: 5m, 1h30m, 90s".to_string());
    }

    Ok(Duration::from_secs(total_seconds))
}

/// Set a reminder after a given duration.
#[poise::command(slash_command)]
pub async fn timer(
    ctx: Context<'_>,
    #[description = "Duration (e.g. 5m, 1h30m, 90s)"] duration_str: String,
    #[description = "Reminder message"] message: String,
) -> Result<(), Error> {
    let duration = match parse_duration(&duration_str) {
        Ok(d) => d,
        Err(e) => {
            ctx.say(format!("❌ Error: {}", e)).await?;
            return Ok(());
        }
    };

    // `const` values are evaluated at compile time and inlined
    const MAX_DURATION: Duration = Duration::from_secs(24 * 60 * 60);
    if duration > MAX_DURATION {
        ctx.say("❌ Maximum duration: 24 hours").await?;
        return Ok(());
    }

    let minutes = duration.as_secs() / 60;
    let seconds = duration.as_secs() % 60;

    ctx.say(format!(
        "⏰ Timer set for **{}m {:02}s**\nMessage: {}",
        minutes, seconds, message
    ))
    .await?;

    // Clone data needed by the spawned task. The task may outlive the
    // current function, so it can't borrow — it must own its data.
    // Arc::clone() is cheap (just increments a counter).
    let channel_id = ctx.channel_id();
    let http = ctx.serenity_context().http.clone();
    let author_id = ctx.author().id;

    // `tokio::spawn` creates an independent async task.
    // `async move` takes ownership of captured variables (channel_id, http, etc.)
    tokio::spawn(async move {
        // .await suspends this task — the thread is free for other work
        sleep(duration).await;

        let reminder = format!("🔔 <@{}> Reminder: **{}**", author_id, message);

        // `if let Err(e)` handles only the error case, ignoring Ok
        if let Err(e) = channel_id.say(&http, reminder).await {
            tracing::error!("Failed to send reminder: {}", e);
        }
    });

    Ok(())
}

/// Start a Pomodoro session (work + break cycle).
#[poise::command(slash_command)]
pub async fn pomodoro(
    ctx: Context<'_>,
    #[description = "Work duration in minutes (default: 25)"] work_minutes: Option<u64>,
    #[description = "Break duration in minutes (default: 5)"] break_minutes: Option<u64>,
) -> Result<(), Error> {
    // .unwrap_or() extracts the value from Some, or uses the default for None
    let work = work_minutes.unwrap_or(25);
    let pause = break_minutes.unwrap_or(5);

    if work == 0 || work > 120 {
        ctx.say("❌ Work duration must be between 1 and 120 minutes")
            .await?;
        return Ok(());
    }

    ctx.say(format!(
        "🍅 **Pomodoro session started!**\n\
        ⏱️ Work: {} minutes\n\
        ☕ Break: {} minutes\n\n\
        Focus! I'll notify you when it's done.",
        work, pause
    ))
    .await?;

    let channel_id = ctx.channel_id();
    let http = ctx.serenity_context().http.clone();
    let author_id = ctx.author().id;

    tokio::spawn(async move {
        // Work phase
        sleep(Duration::from_secs(work * 60)).await;

        let work_done_msg = format!(
            "🍅 <@{}> **Work time is up!**\n\
            Take a {} minute break. You earned it! ☕",
            author_id, pause
        );

        if let Err(e) = channel_id.say(&http, &work_done_msg).await {
            tracing::error!("Pomodoro notification error: {}", e);
            return; // Early return from the spawned task
        }

        // Break phase
        sleep(Duration::from_secs(pause * 60)).await;

        let break_done_msg = format!(
            "🍅 <@{}> **Break is over!**\n\
            Ready for another session? Use `/pomodoro` to start again!",
            author_id
        );

        if let Err(e) = channel_id.say(&http, break_done_msg).await {
            tracing::error!("Break notification error: {}", e);
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
