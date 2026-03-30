//! # Developer Commands
//!
//! ## Rust concepts covered:
//! - Enums and pattern matching
//! - Derive macros (Debug, Clone, Copy)
//! - `impl` blocks for adding methods to types
//! - `const` vs `let` (compile-time vs runtime)
//! - String formatting and manipulation
//! - `if let` for single-pattern matching

use crate::{Context, Error};

/// Enums in Rust can be simple (like C) or hold data per variant.
/// Here we use a simple enum with derived traits:
/// - `Copy`: implicit bitwise copy (only for small, stack-allocated types)
/// - `Clone`: explicit `.clone()` — `Copy` implies `Clone`
/// - `poise::ChoiceParameter`: generates Discord slash command choices
#[derive(Debug, Clone, Copy, poise::ChoiceParameter)]
pub enum Language {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Go,
    C,
    Cpp,
    Java,
    Sql,
    Bash,
    Json,
    Yaml,
    Toml,
}

/// `impl` blocks attach methods and associated functions to a type.
/// Methods take `&self` (immutable borrow) or `&mut self` (mutable borrow).
impl Language {
    fn as_discord_lang(&self) -> &'static str {
        // Exhaustive match — compiler ensures all variants are handled
        match self {
            Language::Rust => "rust",
            Language::Python => "python",
            Language::JavaScript => "javascript",
            Language::TypeScript => "typescript",
            Language::Go => "go",
            Language::C => "c",
            Language::Cpp => "cpp",
            Language::Java => "java",
            Language::Sql => "sql",
            Language::Bash => "bash",
            Language::Json => "json",
            Language::Yaml => "yaml",
            Language::Toml => "toml",
        }
    }
}

/// Share a formatted code snippet with syntax highlighting.
#[poise::command(slash_command)]
pub async fn snippet(
    ctx: Context<'_>,
    #[description = "Programming language"] language: Language,
    #[description = "Code to share"] code: String,
    #[description = "Optional description"] description: Option<String>,
) -> Result<(), Error> {
    let lang_str = language.as_discord_lang();

    // `if let` is a concise alternative to `match` when you only
    // care about one variant (here, Some)
    let header = if let Some(desc) = description {
        format!("**{}**\n", desc)
    } else {
        String::new()
    };

    let formatted = format!(
        "{}```{}\n{}\n```",
        header, lang_str, code
    );

    ctx.say(formatted).await?;
    Ok(())
}

/// Search Rust documentation.
#[poise::command(slash_command)]
pub async fn docs(
    ctx: Context<'_>,
    #[description = "Search term"] query: String,
) -> Result<(), Error> {
    let encoded_query = query.replace(' ', "+");

    const DOCS_BASE_URL: &str = "https://doc.rust-lang.org/std";
    const CRATES_IO_URL: &str = "https://docs.rs";

    let response = format!(
        "**Search for `{}`:**\n\n\
        📚 [Standard library docs]({}/index.html?search={})\n\
        📦 [docs.rs (crates)]({}?search={})\n\
        🔍 [Rust by Example](https://doc.rust-lang.org/rust-by-example/)\n\n\
        Tip: Run `cargo doc --open` to view your project's docs locally!",
        query,
        DOCS_BASE_URL, encoded_query,
        CRATES_IO_URL, encoded_query
    );

    ctx.say(response).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_as_discord_lang() {
        assert_eq!(Language::Rust.as_discord_lang(), "rust");
        assert_eq!(Language::TypeScript.as_discord_lang(), "typescript");
        assert_eq!(Language::Cpp.as_discord_lang(), "cpp");
    }
}
