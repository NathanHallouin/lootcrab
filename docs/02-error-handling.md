# Error Handling

Rust has no exceptions. Instead, it uses `Result<T, E>` and `Option<T>`.

## Option\<T\> — Maybe a Value

Represents a value that might not exist: `Some(value)` or `None`.
This is Rust's replacement for null — the compiler forces you to handle both cases.

```rust
// src/commands/general.rs
pub async fn help(ctx: Context<'_>, command: Option<String>) {
    match command {
        Some(cmd) => { /* user provided a command */ }
        None => { /* no command provided */ }
    }

    // Shorthand: if let (when you only care about one case)
    if let Some(cmd) = command {
        // ...
    }
}

// src/commands/timer.rs
let work = work_minutes.unwrap_or(25);  // Extract or use default
```

### Option Chaining

```rust
// src/services/free_games.rs — chaining Option methods
let slug = game.catalog_ns.mappings
    .as_ref()           // Option<Vec> → Option<&Vec> (borrow without consuming)
    .and_then(|m| m.first())  // Option<&Vec> → Option<&Item> (flatMap)
    .map(|m| m.page_slug.clone())  // Option<&Item> → Option<String>
    .or_else(|| /* try another source */)  // Fallback on None
    .or(game.url_slug)?;  // Final fallback, ? returns None from filter_map
```

## Result\<T, E\> — Success or Error

```rust
// src/commands/timer.rs
fn parse_duration(input: &str) -> Result<Duration, String> {
    if total_seconds == 0 {
        return Err("Invalid duration".to_string());
    }
    Ok(Duration::from_secs(total_seconds))
}

// Usage with match
match parse_duration("5m") {
    Ok(duration) => { /* use duration */ }
    Err(e) => { /* handle error */ }
}
```

## The ? Operator

Propagates errors automatically — returns early on `Err` (or `None` for Option).

```rust
// src/commands/games.rs
pub async fn freegames(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;  // Returns Err early if this fails
    // ...
    ctx.send(reply).await?;  // Same here
    Ok(())
}

// In filter_map, ? on Option returns None (skipping the element)
let promo = game.promotions.as_ref()?;  // Skip games without promotions
```

## Error Conversion

`Box<dyn Error + Send + Sync>` is a trait object that can hold any error type.
The `?` operator automatically converts errors using the `From` trait.

```rust
// src/main.rs
pub type Error = Box<dyn std::error::Error + Send + Sync>;

// .map_err() transforms the error type
config.save().await
    .map_err(|e| format!("Save error: {}", e))?;

// .into() converts between compatible error types
return Err(e.into());
```

## Best Practices

| Do | Don't |
|----|-------|
| Use `?` to propagate errors | Use `.unwrap()` in production code |
| Use `.expect("message")` for unrecoverable errors | Ignore errors with `let _ =` |
| Use `Box<dyn Error>` for application code | Return `String` as error type |
| Use `.unwrap_or()` / `.unwrap_or_default()` | Panic on recoverable errors |
