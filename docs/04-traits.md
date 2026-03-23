# Traits

Traits define shared behavior (like interfaces in other languages).

## Standard Library Traits

```rust
// Clone - explicit duplication
trait Clone {
    fn clone(&self) -> Self;
}

// Debug - debug formatting
trait Debug {
    fn fmt(&self, f: &mut Formatter) -> Result;
}

// Default - default values
trait Default {
    fn default() -> Self;
}

// Display - user-facing formatting
trait Display {
    fn fmt(&self, f: &mut Formatter) -> Result;
}
```

## Implementing Traits

### Using Derive (automatic)

```rust
#[derive(Debug, Clone, Default)]
pub struct PomodoroConfig {
    pub work_duration: u64,
    pub break_duration: u64,
}
// Compiler generates implementations
```

### Manual Implementation

```rust
// src/models/mod.rs
impl Default for PomodoroConfig {
    fn default() -> Self {
        Self {
            work_duration: 25,
            break_duration: 5,
        }
    }
}

// src/services/free_games.rs
impl std::fmt::Display for Store {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Store::Steam => write!(f, "Steam"),
            Store::EpicGames => write!(f, "Epic Games"),
        }
    }
}
```

## The From/Into Traits

Enable type conversions.

```rust
// src/error.rs
impl From<std::env::VarError> for BotError {
    fn from(err: std::env::VarError) -> Self {
        BotError::Config(err.to_string())
    }
}

// Now this works automatically:
let token = std::env::var("TOKEN")?;  // VarError converts to BotError
```

## Trait Bounds

Constrain generic types.

```rust
// T must implement Clone and Debug
fn process<T: Clone + Debug>(item: T) {
    let copy = item.clone();
    println!("{:?}", copy);
}

// Alternative syntax with where
fn process<T>(item: T)
where
    T: Clone + Debug,
{
    // ...
}
```

## Common Patterns

| Pattern | Traits Used |
|---------|-------------|
| Printable for debugging | `Debug` |
| Duplicatable | `Clone` (explicit) or `Copy` (implicit) |
| Comparable | `PartialEq`, `Eq`, `PartialOrd`, `Ord` |
| Hashable (for HashMap keys) | `Hash` + `Eq` |
| Serializable | `Serialize`, `Deserialize` (serde) |
| Error type | `Error` + `Display` + `Debug` |
