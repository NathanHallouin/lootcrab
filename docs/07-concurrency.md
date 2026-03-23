# Concurrency & Thread Safety

## The Problem

Multiple async tasks may need to access the same data. Rust's ownership system prevents data races at compile time.

## Arc - Shared Ownership

`Arc` (Atomic Reference Counting) allows multiple owners of the same data.

```rust
use std::sync::Arc;

// Without Arc - can't share
let http = Http::new();
tokio::spawn(async move {
    http.send().await;  // http moved here
});
// Can't use http anymore!

// With Arc - shared ownership
let http = Arc::new(Http::new());
let http_clone = http.clone();  // Cheap clone (just increments counter)
tokio::spawn(async move {
    http_clone.send().await;
});
// Original http still usable
```

## RwLock - Interior Mutability

`RwLock` allows multiple readers OR one writer.

```rust
use tokio::sync::RwLock;

let config = RwLock::new(Config::default());

// Multiple readers allowed simultaneously
let guard = config.read().await;
println!("{}", guard.channel_id);

// Only one writer at a time (blocks readers)
let mut guard = config.write().await;
guard.channel_id = Some(12345);
```

## Arc<RwLock<T>> Pattern

Combine both for shared mutable state across tasks.

```rust
// src/services/config.rs
#[derive(Clone)]
pub struct ConfigManager {
    config: Arc<RwLock<BotConfig>>,
    path: PathBuf,
}

impl ConfigManager {
    pub async fn get(&self) -> BotConfig {
        // Read lock - multiple readers OK
        self.config.read().await.clone()
    }

    pub async fn set_channel(&self, id: u64) -> Result<(), Error> {
        // Write lock - exclusive access
        {
            let mut config = self.config.write().await;
            config.channel_id = Some(id);
        }  // Lock released here

        self.save().await
    }
}
```

## When to Use What

| Scenario | Solution |
|----------|----------|
| Share read-only data | `Arc<T>` |
| Share data, rarely mutate | `Arc<RwLock<T>>` |
| Share data, frequently mutate | `Arc<Mutex<T>>` |
| Single task, need mutation | `RefCell<T>` (not Send) |
| Counter across tasks | `AtomicU64` |

## Avoiding Deadlocks

```rust
// BAD - holding lock across await
let guard = config.write().await;
do_something().await;  // Still holding lock!
drop(guard);

// GOOD - release lock before await
{
    let mut guard = config.write().await;
    guard.value = 42;
}  // Lock released
do_something().await;
```

## In This Project

```rust
// src/services/scheduler.rs
pub struct FreeGamesScheduler {
    http: Arc<Http>,           // Shared Discord client
    config: ConfigManager,     // Contains Arc<RwLock<BotConfig>>
}

impl FreeGamesScheduler {
    pub async fn run(self) {
        loop {
            // Read config (read lock, released immediately)
            let config = self.config.get().await;

            // Use shared http client
            self.http.send(&channel, &message).await;
        }
    }
}
```

## Key Takeaways

- Rust prevents data races at compile time
- `Arc` for sharing, `RwLock`/`Mutex` for mutation
- Always release locks before `.await`
- Clone `Arc` is cheap (just a counter increment)
