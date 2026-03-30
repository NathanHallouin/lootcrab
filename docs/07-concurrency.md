# Concurrency & Thread Safety

## The Problem

Multiple async tasks may need to access the same data. Rust's type system prevents
data races at compile time — if it compiles, it's thread-safe.

## Arc — Shared Ownership Across Tasks

`Arc` (Atomic Reference Counting) allows multiple owners of the same data.
`.clone()` increments a counter (cheap), `Drop` decrements it (deallocates at zero).

```rust
// src/commands/timer.rs
// Without Arc — can't share across tasks
let http = ctx.serenity_context().http;  // This is already an Arc<Http>
let http_clone = http.clone();           // Just increments the counter

tokio::spawn(async move {
    http_clone.send(&channel, &msg).await;  // Uses the shared client
});
// Original http is still usable here
```

## RwLock — Interior Mutability

`RwLock` allows multiple concurrent readers OR one exclusive writer.

```rust
use tokio::sync::RwLock;

let config = RwLock::new(BotConfig::default());

// Multiple readers allowed simultaneously
let guard = config.read().await;
println!("{}", guard.free_games_hour);
// guard is dropped here, releasing the lock

// Only one writer at a time (blocks all readers)
let mut guard = config.write().await;
guard.free_games_hour = 10;
// guard is dropped here
```

## Arc\<RwLock\<T\>\> — The Key Pattern

Combine both for shared mutable state across async tasks.

```rust
// src/services/config.rs
#[derive(Clone)]  // Clone just clones the Arc (cheap)
pub struct ConfigManager {
    config: Arc<RwLock<BotConfig>>,  // Shared + mutable
    path: PathBuf,                    // Owned, cloned with ConfigManager
}

impl ConfigManager {
    pub async fn get(&self) -> BotConfig {
        // Read lock — multiple tasks can read simultaneously
        self.config.read().await.clone()
    }

    pub async fn set_channel(&self, id: Option<u64>) -> Result<(), Error> {
        {
            // Write lock — exclusive access
            let mut config = self.config.write().await;
            config.free_games_channel_id = id;
        }  // Lock released here — BEFORE the await below

        self.save().await  // save() needs a read lock — no deadlock!
    }
}
```

## Avoiding Deadlocks

**Never hold a lock across an `.await` point.**

```rust
// BAD — holding write lock across await → potential deadlock
let guard = config.write().await;
do_something().await;  // Other tasks can't read or write!
drop(guard);

// GOOD — scope the lock, release before await
{
    let mut guard = config.write().await;
    guard.value = 42;
}  // Lock released
do_something().await;  // Other tasks are free to access
```

## When to Use What

| Scenario | Type | Example in LootCrab |
|----------|------|---------------------|
| Share read-only data | `Arc<T>` | `Arc<Http>` in scheduler |
| Share + occasionally mutate | `Arc<RwLock<T>>` | `ConfigManager` |
| Share + frequently mutate | `Arc<Mutex<T>>` | — |
| Simple counter | `AtomicU64` | — |
| Single-thread mutation | `RefCell<T>` | — (not Send) |

## How It All Connects in LootCrab

```rust
// src/main.rs — setup
let config = ConfigManager::load(path).await;  // Creates Arc<RwLock<BotConfig>>

// Clone for the scheduler (Arc::clone = counter increment)
let scheduler = FreeGamesScheduler::new(http, config.clone());
tokio::spawn(async move { scheduler.run().await; });

// Same config shared with all commands via Data
Ok(Data { start_time: Instant::now(), config })
```

```rust
// src/services/scheduler.rs — reads config every cycle
loop {
    let config = self.config.get().await;  // Read lock, clone, release
    // ...
}

// src/commands/games.rs — writes config from slash command
config.set_free_games_channel(Some(channel_id)).await;  // Write lock, save, release
```

Both the scheduler and the slash commands access the same `ConfigManager` concurrently
— the `RwLock` ensures they never corrupt each other's data.

## Key Takeaways

- Rust prevents data races at **compile time** — no runtime checks needed
- `Arc` for sharing, `RwLock`/`Mutex` for mutation
- Always release locks before `.await` (use scoping with `{ }`)
- `Arc::clone()` is cheap — just a counter increment
- `Send + Sync` traits are checked at compile time for async task safety
