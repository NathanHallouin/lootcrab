# Structs & Enums

## Structs

Group related data together.

```rust
// src/models/mod.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reminder {
    pub id: u64,
    pub message: String,
    pub trigger_at: DateTime<Utc>,
}

impl Reminder {
    // Associated function (constructor)
    pub fn new(id: u64, message: String) -> Self {
        Self { id, message, trigger_at: Utc::now() }
    }

    // Method (takes &self)
    pub fn should_trigger(&self) -> bool {
        Utc::now() >= self.trigger_at
    }
}
```

## Enums

Define a type with multiple variants. Unlike C enums, Rust enums can hold data.

### Simple Enum

```rust
// src/commands/dev.rs
#[derive(Debug, Clone, Copy)]
pub enum Language {
    Rust,
    Python,
    JavaScript,
}

impl Language {
    fn as_str(&self) -> &'static str {
        match self {
            Language::Rust => "rust",
            Language::Python => "python",
            Language::JavaScript => "javascript",
        }
    }
}
```

### Enum with Data

```rust
// src/error.rs
#[derive(Error, Debug)]
pub enum BotError {
    Config(String),       // Holds a String
    RateLimit(u64),       // Holds a number
    Discord(SerenityError), // Holds another error type
}
```

## Pattern Matching

`match` must handle all cases (exhaustive).

```rust
match language {
    Language::Rust => println!("Best language!"),
    Language::Python => println!("Also good"),
    _ => println!("Other"),  // Catch-all
}

// With guards
match value {
    n if n < 0 => println!("Negative"),
    0 => println!("Zero"),
    n => println!("Positive: {}", n),
}
```

### if let - Single Pattern

```rust
// Instead of matching all cases
if let Some(channel_id) = config.channel {
    // Only runs if Some
}

// Equivalent to
match config.channel {
    Some(channel_id) => { /* ... */ }
    None => {}
}
```

## Derive Macros

Auto-generate trait implementations.

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    pub channel_id: Option<u64>,
    pub hour: u32,
}
```

| Derive | What it does |
|--------|--------------|
| `Debug` | Enables `{:?}` formatting |
| `Clone` | Enables `.clone()` |
| `Copy` | Implicit copy (small types only) |
| `Default` | Creates default values |
| `PartialEq` | Enables `==` comparison |
| `Serialize` | JSON/etc serialization (serde) |
| `Deserialize` | JSON/etc deserialization (serde) |
