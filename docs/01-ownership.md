# Ownership, Borrowing & Lifetimes

## The Problem Rust Solves

In C/C++, memory management is manual and error-prone (use-after-free, double-free).
In Java/Python, a garbage collector handles memory but with performance costs.

Rust uses an **ownership** system verified at compile time.

## The Three Rules

1. Each value has one **owner**
2. There can only be **one owner** at a time
3. When the owner goes out of scope, the value is **dropped**

## Examples in This Project

### &str vs String

```rust
// src/commands/general.rs
fn get_command_help(command: &str) -> &'static str {
    // `command` is borrowed - we don't take ownership
    // &str means "reference to a string slice"
    match command {
        "ping" => "...",  // String literals have 'static lifetime
        _ => "...",
    }
}
```

| Type | Ownership | Allocation |
|------|-----------|------------|
| `&str` | Borrowed (reference) | Usually stack or static |
| `String` | Owned | Heap |

```rust
let s1: &str = "hello";           // String slice (borrowed)
let s2: String = s1.to_string();  // New allocation (owned)
let s3: &str = &s2;               // Borrow from s2
```

## Lifetimes

Lifetimes tell the compiler how long references are valid.

```rust
// 'a is a lifetime - the returned reference lives as long as s
fn get_first_word<'a>(s: &'a str) -> &'a str {
    &s[..5]
}

// 'static - lives for the entire program
fn constant() -> &'static str {
    "hello"  // String literals are 'static
}
```

### In This Project

```rust
// src/main.rs
pub type Context<'a> = poise::Context<'a, Data, Error>;
// The Context borrows data and can't outlive it
```

## Key Takeaways

- Rust prevents memory bugs at compile time
- References (`&`) borrow data without taking ownership
- Lifetimes ensure references are always valid
- The borrow checker is your friend, not your enemy
