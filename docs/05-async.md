# Async Programming

## Why Async?

A Discord bot is I/O-bound: it waits for network responses, user input, timers.
Async allows suspending a task while waiting, freeing the thread for other work —
without the overhead of OS threads.

## Key Concepts

| Concept | Description |
|---------|-------------|
| `async fn` | Returns a `Future` — doesn't execute until awaited |
| `.await` | Suspends execution until the Future completes |
| `Future` | A value that will be available later |
| Runtime | Executes futures (we use Tokio) |

## The Tokio Runtime

```rust
// src/main.rs
#[tokio::main]  // Macro that creates a multi-threaded Tokio runtime
async fn main() -> Result<(), Error> {
    // All async code runs inside this runtime
}
```

`#[tokio::main]` expands to roughly:

```rust
fn main() {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        // your async main body
    })
}
```

## Spawning Independent Tasks

`tokio::spawn` creates a task that runs independently in the background.

```rust
// src/commands/timer.rs
// Clone data before the move — spawned tasks must own their data
let channel_id = ctx.channel_id();
let http = ctx.serenity_context().http.clone();  // Arc::clone (cheap)

// `async move` takes ownership of captured variables
tokio::spawn(async move {
    sleep(duration).await;  // Suspends this task, not the whole thread
    channel_id.say(&http, reminder).await;
});
// Function returns immediately — timer runs in background
```

## Parallel Execution with tokio::join!

```rust
// src/services/free_games.rs
// Both requests are in-flight simultaneously
let (epic_result, gamerpower_result) = tokio::join!(
    fetch_epic_free_games(),      // Starts immediately
    fetch_gamerpower_games()      // Starts immediately
);
// Continues when BOTH are done
```

Compare with sequential (slower):

```rust
let epic = fetch_epic_free_games().await;      // Wait...
let steam = fetch_gamerpower_games().await;    // Then wait again...
```

## Async File I/O

```rust
// src/services/config.rs — tokio::fs is the async version of std::fs
let content = tokio::fs::read_to_string(&path).await?;
tokio::fs::write(&path, json).await?;
tokio::fs::create_dir_all(parent).await?;
```

## Deferring Long Operations

```rust
// src/commands/games.rs
pub async fn freegames(ctx: Context<'_>) -> Result<(), Error> {
    // Discord requires a response within 3 seconds.
    // defer() sends "bot is thinking..." and gives us 15 minutes.
    ctx.defer().await?;

    let games = fetch_all_free_games().await;  // May take several seconds
    ctx.send(reply).await?;
    Ok(())
}
```

## Long-Running Loops

```rust
// src/services/scheduler.rs
pub async fn run(self) {
    loop {
        let config = self.config.get().await;  // Read current config
        let wait = Self::duration_until_next_run(config.hour, config.minute);

        sleep(wait).await;  // Suspend for hours without blocking

        self.send_notification(channel_id).await;
    }
}
```

## Async Tests

```rust
// src/services/config.rs
#[tokio::test]  // Creates a Tokio runtime for the test
async fn test_config_save_load() {
    let manager = ConfigManager::load(path).await;
    manager.set_free_games_channel(Some(12345)).await.unwrap();
    // ...
}
```

## Common Pitfalls

| Issue | Solution |
|-------|----------|
| Can't hold `&self` across `.await` | Clone data or use `Arc` |
| Blocking code in async | Use `tokio::task::spawn_blocking` |
| Future is not `Send` | Ensure all captured data is `Send` |
| Holding a lock across `.await` | Scope the lock guard with `{ }` |
