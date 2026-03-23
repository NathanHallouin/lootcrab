# DevGamer Bot 🎮🦀

A Discord bot built in Rust, designed for learning the language while being useful for developers and gamers.

## Features

### General Commands
- `/ping` - Check bot latency
- `/help [command]` - Display help
- `/uptime` - Time since bot started

### Developer Tools
- `/snippet <language> <code>` - Share formatted code snippets
- `/docs <query>` - Links to Rust documentation

### Timers
- `/timer <duration> <message>` - Set a reminder (e.g., `/timer 5m Coffee break`)
- `/pomodoro [work] [break]` - Start a Pomodoro session

### Gaming
- `/freegames` - Display current free games (Epic Games + Steam)
- `/freegames-setup #channel [hour] [minute]` - Configure automatic notifications
- `/freegames-status` - Show current configuration

## Installation

### Prerequisites
- [Rust](https://rustup.rs/) (1.70+)
- A [Discord bot](https://discord.com/developers/applications)

### Setup

1. **Clone and configure**
```bash
cd devgamer-bot
cp .env.example .env
# Edit .env and add your DISCORD_TOKEN
```

2. **Build and run**
```bash
cargo run
```

3. **Invite the bot to your server**
   - Go to Discord Developer Portal
   - OAuth2 → URL Generator
   - Scopes: `bot`, `applications.commands`
   - Permissions: `Send Messages`, `Embed Links`, `Mention Everyone`

4. **Configure free games notifications** (in Discord)
```
/freegames-setup #gaming-deals 9 0
```

## Learning Rust

This project is structured to teach Rust concepts progressively. See the [docs/](docs/) folder for detailed explanations:

- [01-ownership.md](docs/01-ownership.md) - Ownership, borrowing, and lifetimes
- [02-error-handling.md](docs/02-error-handling.md) - Result, Option, and the ? operator
- [03-structs-enums.md](docs/03-structs-enums.md) - Data structures and pattern matching
- [04-traits.md](docs/04-traits.md) - Traits, derive macros, and generics
- [05-async.md](docs/05-async.md) - Async/await with Tokio
- [06-modules.md](docs/06-modules.md) - Code organization and visibility
- [07-concurrency.md](docs/07-concurrency.md) - Arc, RwLock, and thread-safe sharing

## Project Structure

```
src/
├── main.rs              # Entry point - async runtime, setup, scheduler
├── error.rs             # Custom errors - thiserror
├── commands/
│   ├── mod.rs           # Module exports
│   ├── general.rs       # Basic commands
│   ├── dev.rs           # Developer tools
│   ├── timer.rs         # Timers and Pomodoro
│   └── games.rs         # Free games
├── models/              # Data structures
├── services/
│   ├── config.rs        # Persistent config (Arc<RwLock>)
│   ├── free_games.rs    # Epic/Steam API
│   └── scheduler.rs     # Scheduled tasks
└── utils/               # Helpers
```

## Development

```bash
cargo clippy    # Linter
cargo fmt       # Formatter
cargo test      # Run tests
cargo doc --open # View documentation
```

## Roadmap

- [x] Free games notifications (Epic Games + Steam)
- [ ] SQLite persistence
- [ ] GitHub integration (PR notifications)
- [ ] Detailed game stats

## License

MIT
