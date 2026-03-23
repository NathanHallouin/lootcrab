//! # Utilitaires
//!
//! ## Concepts Rust illustrés :
//! - Fonctions pures et réutilisables
//! - Traits personnalisés

/// Formate une durée de manière lisible.
///
/// ## Concept Rust : Pattern matching multiple
pub fn format_duration(seconds: u64) -> String {
    match seconds {
        0 => "0 secondes".to_string(),
        1 => "1 seconde".to_string(),
        s if s < 60 => format!("{} secondes", s),
        s if s < 3600 => {
            let mins = s / 60;
            let secs = s % 60;
            if secs == 0 {
                format!("{} minute{}", mins, if mins > 1 { "s" } else { "" })
            } else {
                format!("{}m {:02}s", mins, secs)
            }
        }
        s => {
            let hours = s / 3600;
            let mins = (s % 3600) / 60;
            format!("{}h {:02}m", hours, mins)
        }
    }
}

/// Tronque une string à une longueur maximale.
///
/// ## Concept Rust : &str et String
/// Cette fonction prend un &str (borrowed) et retourne une Cow
/// pour éviter une allocation si non nécessaire.
pub fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        // ## Concept : char_indices pour UTF-8
        // On ne peut pas simplement couper à un byte offset
        // car ça pourrait couper un caractère UTF-8 en deux
        let mut end = 0;
        for (i, _) in s.char_indices() {
            if i >= max_len - 3 {
                break;
            }
            end = i;
        }
        format!("{}...", &s[..=end])
    }
}

/// Échappe les caractères spéciaux Discord.
pub fn escape_discord(s: &str) -> String {
    s.replace('*', r"\*")
        .replace('_', r"\_")
        .replace('`', r"\`")
        .replace('~', r"\~")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(0), "0 secondes");
        assert_eq!(format_duration(1), "1 seconde");
        assert_eq!(format_duration(45), "45 secondes");
        assert_eq!(format_duration(60), "1 minute");
        assert_eq!(format_duration(90), "1m 30s");
        assert_eq!(format_duration(3600), "1h 00m");
        assert_eq!(format_duration(3665), "1h 01m");
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world this is long", 10), "hello w...");
    }

    #[test]
    fn test_escape_discord() {
        assert_eq!(escape_discord("**bold**"), r"\*\*bold\*\*");
        assert_eq!(escape_discord("_italic_"), r"\_italic\_");
    }
}
