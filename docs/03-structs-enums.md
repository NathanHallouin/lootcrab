# Structs & Enums

## Structs

Group related data together. Fields are private by default.

```rust
// src/services/config.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotConfig {
    pub free_games_channel_id: Option<u64>,  // pub = accessible from outside
    pub free_games_hour: u32,
}

impl BotConfig {
    // No self parameter → associated function, called with ::
    fn default() -> Self { /* ... */ }
}
```

```rust
// src/services/free_games.rs
#[derive(Debug, Clone)]
pub struct FreeGame {
    pub title: String,
    pub store: Store,
    pub url: String,
    pub original_price: Option<String>,
    pub end_date: Option<DateTime<Utc>>,
    pub image_url: Option<String>,
}
```

### Struct Initialization

```rust
// All fields must be set — no partial initialization
FreeGame {
    title: game.title,
    store: Store::EpicGames,
    url,  // Shorthand: field name matches variable name
    original_price: None,
    end_date: Some(current_offer.end_date),
    image_url,
}

// Struct update syntax: fill remaining fields from another instance
FrameworkOptions {
    commands: vec![...],
    ..Default::default()  // All other fields use defaults
}
```

## Enums

Enums define a type with multiple variants. Unlike C enums, Rust enums can hold data.

### Simple Enum

```rust
// src/services/free_games.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Store {
    Steam,
    EpicGames,
}

impl Store {
    pub fn emoji(&self) -> &'static str {
        match self {
            Store::Steam => "🎮",
            Store::EpicGames => "🎁",
        }
    }
}
```

### Enum as Slash Command Choice

```rust
// src/commands/dev.rs — poise generates Discord choices from enum variants
#[derive(Debug, Clone, Copy, poise::ChoiceParameter)]
pub enum Language {
    Rust,
    Python,
    JavaScript,
    // ...
}
```

## Pattern Matching

`match` must be exhaustive — the compiler ensures all cases are handled.

```rust
// Match with guards
match remaining.num_days() {
    d if d > 0 => format!("{}d", d),
    _ => "Last hours!".to_string(),
}

// Match on tuple of Options
match (&a.end_date, &b.end_date) {
    (Some(a_date), Some(b_date)) => a_date.cmp(b_date),
    (Some(_), None) => Ordering::Less,
    (None, Some(_)) => Ordering::Greater,
    (None, None) => Ordering::Equal,
}
```

### if let — Single Pattern

```rust
// Only handle one variant
if let Some(image_url) = &game.image_url {
    embed = embed.image(image_url);
}

// Equivalent but more verbose:
match &game.image_url {
    Some(image_url) => { embed = embed.image(image_url); }
    None => {}
}
```

## Derive Macros

Auto-generate trait implementations at compile time.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotConfig { /* ... */ }
```

| Derive | Effect |
|--------|--------|
| `Debug` | Enables `{:?}` formatting |
| `Clone` | Enables explicit `.clone()` |
| `Copy` | Enables implicit bitwise copy (small stack types only) |
| `Default` | Creates default values via `Default::default()` |
| `PartialEq`, `Eq` | Enables `==` comparison |
| `Serialize` | JSON serialization (serde) |
| `Deserialize` | JSON deserialization (serde) |
