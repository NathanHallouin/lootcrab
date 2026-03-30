# Modules & Code Organization

## Module Structure

```
src/
├── main.rs           # Declares: mod commands; mod services;
├── commands/
│   ├── mod.rs        # Declares: pub mod general; pub mod timer; ...
│   ├── general.rs
│   ├── dev.rs
│   ├── timer.rs
│   └── games.rs
└── services/
    ├── mod.rs        # Declares: pub mod config; pub mod free_games; ...
    ├── config.rs
    ├── free_games.rs
    └── scheduler.rs
```

## Declaring Modules

```rust
// src/main.rs
mod commands;    // Looks for commands/mod.rs
mod services;    // Looks for services/mod.rs

// src/commands/mod.rs
pub mod general;   // Public — accessible from main.rs and siblings
pub mod timer;
pub mod dev;
pub mod games;
```

## Visibility

| Keyword | Scope |
|---------|-------|
| (none) | Private to current module |
| `pub` | Public everywhere |
| `pub(crate)` | Public within this crate only |
| `pub(super)` | Public to parent module only |

```rust
pub struct FreeGame {
    pub title: String,      // Accessible from anywhere
    pub store: Store,       // Public field
}

impl FreeGame {
    pub fn new() -> Self { }   // Public constructor
    fn validate(&self) { }     // Private — only usable within this module
}
```

## Importing with `use`

```rust
// Absolute path (from crate root)
use crate::services::config::ConfigManager;
use crate::{Context, Error};  // Multiple imports from same path

// Relative path (from parent module)
use super::config::ConfigManager;
use super::free_games::{fetch_all_free_games, FreeGame, Store};

// Rename imports to avoid conflicts or improve clarity
use poise::serenity_prelude as serenity;

// Multiple items from the same path
use poise::serenity_prelude::{ChannelId, Colour, CreateEmbed, Http};
```

## In This Project

```rust
// src/main.rs — declares top-level modules
mod commands;
mod services;
use services::config::ConfigManager;

// src/commands/games.rs — imports from sibling modules
use crate::services::free_games::{fetch_all_free_games, FreeGame, Store};
use crate::{Context, Error};  // From main.rs (crate root)

// src/services/scheduler.rs — relative imports within services/
use super::config::ConfigManager;
use super::free_games::{fetch_all_free_games, FreeGame, Store};
```

## Best Practices

- Keep `mod.rs` files clean — only module declarations, no implementation
- Group related functionality in the same module directory
- Use `pub` sparingly — only expose what's needed by other modules
- Prefer `use crate::` for absolute paths, `use super::` for relative
