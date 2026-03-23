# Async Programming

## Why Async?

A Discord bot handles many I/O operations (network requests). Async allows suspending a task while waiting, freeing the thread for other work.

## Key Concepts

| Concept | Description |
|---------|-------------|
| `async fn` | Returns a `Future` instead of the value directly |
| `await` | Suspends execution until the Future is ready |
| `Future` | A value that will be available later |
| Runtime | Executes futures (we use Tokio) |

## Basic Usage

```rust
// src/main.rs
#[tokio::main]  // Sets up the Tokio runtime
async fn main() -> Result<(), Error> {
    // async code here
}

// src/commands/timer.rs
async fn timer(...) {
    // await suspends until sleep completes
    tokio::time::sleep(duration).await;

    // Continue after sleep
    send_message().await;
}
```

## Spawning Tasks

`tokio::spawn` creates an independent task that runs in the background.

```rust
// src/main.rs
tokio::spawn(async move {
    scheduler.run().await;  // Runs forever in background
});

// Main function continues immediately
// The spawned task runs concurrently
```

## Parallel Execution

```rust
// src/services/free_games.rs

// Sequential (slow)
let epic = fetch_epic_games().await;
let steam = fetch_steam_games().await;

// Parallel (fast) - both requests run simultaneously
let (epic, steam) = tokio::join!(
    fetch_epic_games(),
    fetch_steam_games()
);
```

## Async File I/O

```rust
// src/services/config.rs
use tokio::fs;

// Read file asynchronously
let content = fs::read_to_string(&path).await?;

// Write file asynchronously
fs::write(&path, json).await?;
```

## Common Patterns

### Defer for Long Operations

```rust
// src/commands/games.rs
pub async fn freegames(ctx: Context<'_>) -> Result<(), Error> {
    // Show "typing..." indicator while working
    ctx.defer().await?;

    // Now do slow work
    let games = fetch_all_free_games().await;
    ctx.say(format_games(&games)).await?;
    Ok(())
}
```

### Timeouts

```rust
use tokio::time::{timeout, Duration};

match timeout(Duration::from_secs(10), fetch_data()).await {
    Ok(result) => result?,
    Err(_) => return Err("Request timed out".into()),
}
```

### Periodic Tasks

```rust
// src/services/scheduler.rs
loop {
    let wait = calculate_next_run();
    tokio::time::sleep(wait).await;

    do_scheduled_task().await;
}
```

## Gotchas

| Issue | Solution |
|-------|----------|
| Can't use `&self` across await | Clone data or use `Arc` |
| Blocking code in async | Use `tokio::task::spawn_blocking` |
| Future not Send | Ensure all captured data is Send |
