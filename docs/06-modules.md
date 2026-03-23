# Modules & Code Organization

## Module Structure

```
src/
├── main.rs           # Declares modules: mod commands;
├── commands/
│   ├── mod.rs        # Declares submodules: pub mod general;
│   ├── general.rs    # Implementation
│   └── timer.rs
└── services/
    ├── mod.rs
    └── config.rs
```

## Declaring Modules

```rust
// src/main.rs
mod commands;    // Looks for commands/mod.rs or commands.rs
mod services;
mod utils;

// src/commands/mod.rs
pub mod general;   // Public - accessible from outside
pub mod timer;
mod internal;      // Private - only accessible within commands/
```

## Visibility

| Keyword | Visibility |
|---------|------------|
| (none) | Private to current module |
| `pub` | Public everywhere |
| `pub(crate)` | Public within the crate only |
| `pub(super)` | Public to parent module |

```rust
pub struct Config {
    pub channel_id: u64,      // Public field
    password: String,          // Private field
}

impl Config {
    pub fn new() -> Self { }   // Public method
    fn validate(&self) { }     // Private method
}
```

## Importing with use

```rust
// Absolute path (from crate root)
use crate::services::config::ConfigManager;

// Relative path
use super::utils::format_duration;  // Parent module
use self::helpers::parse;           // Current module

// Multiple imports
use std::collections::{HashMap, HashSet};

// Rename imports
use poise::serenity_prelude as serenity;

// Re-export
pub use crate::error::BotError;  // Makes BotError available from this module
```

## Best Practices

### Keep mod.rs Clean

```rust
// src/commands/mod.rs
//! Command modules for the Discord bot.

pub mod dev;
pub mod games;
pub mod general;
pub mod timer;

// Don't put implementation here, just declarations
```

### Group Related Functionality

```
services/
├── mod.rs
├── config.rs      # Configuration management
├── free_games.rs  # Free games API
└── scheduler.rs   # Task scheduling
```

### Use Prelude Pattern for Common Imports

```rust
// src/prelude.rs
pub use crate::error::{Error, Result};
pub use crate::Context;

// Then in other files:
use crate::prelude::*;
```

## In This Project

```rust
// src/main.rs
mod commands;
mod error;
mod models;
mod services;
mod utils;

use services::config::ConfigManager;
use services::scheduler::FreeGamesScheduler;

// src/commands/games.rs
use crate::services::free_games::{fetch_all_free_games, FreeGame, Store};
use crate::{Context, Error};
```
