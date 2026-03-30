# LootCrab 🦀

A Discord bot that hunts free games and helps developers — built in Rust as a learning project.

## Features

### Free Games
- `/freegames` — Display current free games with rich embeds (Epic Games + Steam via GamerPower)
- `/freegames-setup #channel [hour] [minute]` — Configure daily automatic notifications
- `/freegames-status` — Show current notification configuration

### Developer Tools
- `/snippet <language> <code>` — Share formatted code snippets with syntax highlighting
- `/docs <query>` — Quick links to Rust documentation

### Timers
- `/timer <duration> <message>` — Set a reminder (e.g., `/timer 5m Coffee break`)
- `/pomodoro [work] [break]` — Start a Pomodoro session (default: 25min work, 5min break)

### General
- `/ping` — Check bot latency
- `/help [command]` — Display help
- `/uptime` — Time since bot started

## Setup

### Prerequisites
- [Rust](https://rustup.rs/) (edition 2024)
- A [Discord bot application](https://discord.com/developers/applications)

### Install & Run

```bash
git clone <repo-url> && cd lootcrab
cp .env.example .env
# Edit .env: add your DISCORD_TOKEN and DEV_GUILD_ID
cargo run
```

### Invite the Bot

1. Go to [Discord Developer Portal](https://discord.com/developers/applications)
2. OAuth2 → URL Generator
3. Scopes: `bot`, `applications.commands`
4. Permissions: `Send Messages`, `Embed Links`
5. Open the generated URL and select your server

### Configure Notifications (in Discord)

```
/freegames-setup #gaming-deals 9 0
```

## Project Structure

```
src/
├── main.rs              # Entry point, async runtime, framework setup
├── commands/
│   ├── mod.rs           # Module declarations
│   ├── general.rs       # ping, help, uptime
│   ├── dev.rs           # snippet, docs
│   ├── timer.rs         # timer, pomodoro
│   └── games.rs         # freegames, freegames-setup, freegames-status
└── services/
    ├── mod.rs           # Module declarations
    ├── config.rs        # Persistent JSON config (Arc<RwLock>)
    ├── free_games.rs    # Epic Games + GamerPower API clients
    └── scheduler.rs     # Daily notification scheduler
```

## Rust Concepts Covered

This project demonstrates key Rust concepts in a real-world application. Each source file has inline documentation, and the [docs/](docs/) folder has detailed explanations:

| Concept | Where | Doc |
|---------|-------|-----|
| Ownership, borrowing, lifetimes | `timer.rs` (clone before move), `general.rs` (&str vs String) | [01-ownership](docs/01-ownership.md) |
| Error handling (Result, Option, ?) | `timer.rs` (parse_duration), `games.rs` (? chaining) | [02-error-handling](docs/02-error-handling.md) |
| Structs, enums, pattern matching | `free_games.rs` (Store enum), `config.rs` (BotConfig) | [03-structs-enums](docs/03-structs-enums.md) |
| Traits (Display, Default, Debug) | `free_games.rs` (Display for Store), `config.rs` (Default) | [04-traits](docs/04-traits.md) |
| Async/await, tokio::spawn, join! | `timer.rs` (spawn), `free_games.rs` (join!), `scheduler.rs` (loop) | [05-async](docs/05-async.md) |
| Modules and visibility | `mod.rs` files, `use crate::`, `pub` | [06-modules](docs/06-modules.md) |
| Concurrency (Arc, RwLock) | `config.rs` (Arc\<RwLock\>), `scheduler.rs` (shared state) | [07-concurrency](docs/07-concurrency.md) |

Additional concepts in the code: iterators (`filter_map`, `flat_map`, `any`, `collect`), closures (`async move`), serde attributes (`rename_all`, `default`), builder pattern (embed construction), type aliases, const vs let, derive macros.

## Development

```bash
cargo test      # Run tests
cargo clippy    # Linter
cargo fmt       # Formatter
cargo doc --open # View generated documentation
```

## License

MIT
