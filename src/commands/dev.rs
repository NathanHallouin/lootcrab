//! # Commandes pour développeurs
//!
//! ## Concepts Rust illustrés :
//! - Enums avec données
//! - Derive macros (Debug, Clone, Copy)
//! - String manipulation

use crate::{Context, Error};

/// Langages supportés pour la coloration syntaxique.
///
/// ## Concept Rust : Enums
/// Les enums Rust peuvent contenir des données (comme des variants algébriques).
/// Ici on utilise un enum simple pour les langages supportés.
///
/// ## Concept : Derive macros
/// `#[derive(...)]` génère automatiquement l'implémentation de traits.
/// - `Debug` : Permet d'afficher avec `{:?}`
/// - `Clone` : Permet de dupliquer la valeur
/// - `Copy` : Permet la copie implicite (pour les petits types)
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

impl Language {
    /// Retourne l'identifiant pour le bloc de code Discord.
    ///
    /// ## Concept : impl block
    /// `impl` permet d'ajouter des méthodes à un type.
    fn as_discord_lang(&self) -> &'static str {
        // ## Concept : self
        // `&self` est une référence immuable vers l'instance
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

/// Commande snippet - Partage un snippet de code formaté.
///
/// ## Concept Rust : Paramètres de commande
/// Les paramètres sont extraits automatiquement par poise
/// grâce aux attributs `#[description]`.
#[poise::command(slash_command, prefix_command)]
pub async fn snippet(
    ctx: Context<'_>,
    #[description = "Langage de programmation"] language: Language,
    #[description = "Le code à partager"] code: String,
    #[description = "Description optionnelle"] description: Option<String>,
) -> Result<(), Error> {
    // ## Concept : String formatting
    // On construit le message avec les backticks Discord

    let lang_str = language.as_discord_lang();

    // ## Concept : if let pour Option
    // Alternative plus concise à match pour un seul cas
    let header = if let Some(desc) = description {
        format!("**{}**\n", desc)
    } else {
        String::new()
    };

    // ## Concept : Raw string literals
    // Pas nécessaire ici mais utile pour les regex ou strings complexes
    let formatted = format!(
        "{}```{}\n{}\n```",
        header, lang_str, code
    );

    ctx.say(formatted).await?;
    Ok(())
}

/// Commande docs - Recherche dans la documentation Rust.
///
/// ## Concept Rust : String slices et manipulation
#[poise::command(slash_command, prefix_command)]
pub async fn docs(
    ctx: Context<'_>,
    #[description = "Terme à rechercher"] query: String,
) -> Result<(), Error> {
    // ## Concept : URL encoding
    // On encode la query pour l'URL (simplifié ici)
    let encoded_query = query.replace(' ', "+");

    // ## Concept : const vs let
    // `const` pour les valeurs connues à la compilation
    const DOCS_BASE_URL: &str = "https://doc.rust-lang.org/std";
    const CRATES_IO_URL: &str = "https://docs.rs";

    let response = format!(
        "**Recherche pour `{}`:**\n\n\
        📚 [Documentation standard]({}/index.html?search={})\n\
        📦 [docs.rs (crates)]({}?search={})\n\
        🔍 [Rust by Example](https://doc.rust-lang.org/rust-by-example/)\n\n\
        💡 Tip: Utilise `cargo doc --open` pour la doc de ton projet!",
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
