# Error Handling

Rust doesn't have exceptions. Instead, it uses `Result<T, E>` and `Option<T>` types.

## Option<T> - Optional Values

Represents a value that might not exist: `Some(value)` or `None`.

```rust
// src/commands/general.rs
pub async fn help(
    ctx: Context<'_>,
    command: Option<String>,  // Can be Some(value) or None
) {
    match command {
        Some(cmd) => { /* use cmd */ }
        None => { /* no command provided */ }
    }

    // Or with if let
    if let Some(cmd) = command {
        // ...
    }
}
```

## Result<T, E> - Success or Error

```rust
// src/commands/timer.rs
fn parse_duration(input: &str) -> Result<Duration, String> {
    if invalid {
        return Err("Error message".to_string());
    }
    Ok(Duration::from_secs(seconds))
}

// Usage with match
match parse_duration("5m") {
    Ok(duration) => { /* use duration */ }
    Err(e) => { /* handle error */ }
}
```

## The ? Operator

Propagates errors automatically - returns early if `Err`.

```rust
async fn example() -> Result<(), Error> {
    // Without ?
    let result = some_operation();
    let value = match result {
        Ok(v) => v,
        Err(e) => return Err(e.into()),
    };

    // With ? (equivalent, much cleaner)
    let value = some_operation()?;

    Ok(())
}
```

## Custom Errors with thiserror

```rust
// src/error.rs
#[derive(Error, Debug)]
pub enum BotError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Discord error: {0}")]
    Discord(#[from] serenity::Error),  // Auto-convert with #[from]

    #[error("Rate limited: retry in {0} seconds")]
    RateLimit(u64),
}
```

## Best Practices

| Do | Don't |
|----|-------|
| Use `?` to propagate errors | Use `.unwrap()` in production code |
| Use `expect("message")` for programmer errors | Ignore errors with `let _ =` |
| Define custom error types | Return `String` as error type |
| Use `anyhow` for applications | Panic on recoverable errors |
