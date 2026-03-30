# Ownership, Borrowing & Lifetimes

## The Problem Rust Solves

In C/C++, memory management is manual and error-prone (use-after-free, double-free).
In Java/Python, a garbage collector handles memory but with performance costs.

Rust uses an **ownership** system verified at compile time — zero runtime cost.

## The Three Rules

1. Each value has exactly one **owner**
2. There can only be **one owner** at a time
3. When the owner goes out of scope, the value is **dropped** (deallocated)

## Where This Shows Up in LootCrab

### &str vs String

```rust
// src/commands/general.rs
fn get_command_help(command: &str) -> &'static str {
    // `command` is borrowed — we read it without taking ownership
    // Return value is &'static str — string literals baked into the binary
    match command {
        "ping" => "Sends a pong to check the bot's latency.",
        _ => "Unknown command.",
    }
}
```

| Type | Ownership | Allocation |
|------|-----------|------------|
| `&str` | Borrowed (reference) | Usually stack or static |
| `String` | Owned | Heap |

### Clone and Move

```rust
// src/commands/timer.rs — cloning before a move
let http = ctx.serenity_context().http.clone();  // Clone the Arc
let author_id = ctx.author().id;                 // Copy (u64 is Copy)

tokio::spawn(async move {
    // `move` transfers ownership of http, author_id into this task.
    // Without clone, the original would be moved and unusable.
    channel_id.say(&http, reminder).await;
});
```

### Borrowing with &

```rust
// src/commands/games.rs
fn build_game_embed(game: &FreeGame) -> CreateEmbed {
    // &FreeGame: we borrow the game, the caller keeps ownership.
    // We can read game.title, game.url, etc. but can't modify them.
    // ...
}
```

## Lifetimes

Lifetimes tell the compiler how long references are valid.

```rust
// src/main.rs
pub type Context<'a> = poise::Context<'a, Data, Error>;
// 'a means: the Context can't outlive the Data it references.

// src/commands/general.rs
fn get_command_help(command: &str) -> &'static str {
    // &'static str: lives for the entire program (string literals)
}
```

## Key Takeaways

- Rust prevents memory bugs at compile time — no garbage collector needed
- References (`&`) borrow data without taking ownership
- `Clone` creates a deep copy, `Copy` creates an implicit bitwise copy
- `move` in closures transfers ownership of captured variables
- Lifetimes ensure references never outlive the data they point to
