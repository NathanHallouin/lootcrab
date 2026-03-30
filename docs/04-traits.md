# Traits

Traits define shared behavior — similar to interfaces in other languages, but more powerful.

## Standard Library Traits

```rust
trait Clone {
    fn clone(&self) -> Self;        // Explicit duplication
}

trait Display {
    fn fmt(&self, f: &mut Formatter) -> Result;  // User-facing: {}
}

trait Debug {
    fn fmt(&self, f: &mut Formatter) -> Result;  // Developer-facing: {:?}
}

trait Default {
    fn default() -> Self;           // Create a default value
}
```

## Implementing Traits

### With Derive (automatic)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Store { Steam, EpicGames }
// The compiler generates all 5 trait implementations
```

### Manual Implementation

```rust
// src/services/free_games.rs — Display for user-facing output
impl std::fmt::Display for Store {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Store::Steam => write!(f, "Steam"),
            Store::EpicGames => write!(f, "Epic Games"),
        }
    }
}
// Now you can: format!("{}", Store::Steam) → "Steam"
//         and: Store::Steam.to_string()    → "Steam"

// src/services/config.rs — Default for sensible initial values
impl Default for BotConfig {
    fn default() -> Self {
        Self {
            free_games_channel_id: None,
            free_games_hour: 9,
            free_games_minute: 0,
        }
    }
}
// Now you can: BotConfig::default()
//         and: serde_json::from_str(&s).unwrap_or_default()
```

## Trait Objects — Dynamic Dispatch

When you need to hold "any type that implements a trait":

```rust
// src/main.rs
pub type Error = Box<dyn std::error::Error + Send + Sync>;
// Box<dyn Trait> = a pointer to any type implementing that trait.
// `dyn` = dynamic dispatch (resolved at runtime via vtable).
// `Send + Sync` = safe to use across threads (required for async).
```

## Trait Bounds — Constraining Generics

```rust
// T must implement Clone and Debug
fn process<T: Clone + Debug>(item: T) {
    let copy = item.clone();
    println!("{:?}", copy);
}

// Alternative with `where` clause (cleaner for many bounds)
fn process<T>(item: T)
where
    T: Clone + Debug + Send + Sync,
{
    // ...
}
```

## Auto Traits: Send and Sync

Rust uses `Send` and `Sync` to guarantee thread safety at compile time:

| Trait | Meaning |
|-------|---------|
| `Send` | Can be transferred to another thread |
| `Sync` | Can be referenced from multiple threads |

Most types are automatically `Send + Sync`. Notable exceptions:
- `Rc` (use `Arc` instead)
- `RefCell` (use `RwLock` or `Mutex` instead)

```rust
// src/main.rs — Error must be Send + Sync for use across async tasks
pub type Error = Box<dyn std::error::Error + Send + Sync>;
```

## Common Trait Patterns in This Project

| Pattern | Traits | Example |
|---------|--------|---------|
| Debugging | `Debug` | All structs and enums |
| Cloning | `Clone`, `Copy` | `Store` (Copy), `FreeGame` (Clone) |
| Comparison | `PartialEq`, `Eq` | `Store` for `==` in `.filter()` |
| Serialization | `Serialize`, `Deserialize` | `BotConfig` ↔ JSON |
| User display | `Display` | `Store` for embed text |
| Default values | `Default` | `BotConfig::default()` |
| Error types | `Error + Display + Debug` | `Box<dyn Error>` |
