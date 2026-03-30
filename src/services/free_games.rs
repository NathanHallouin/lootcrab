//! # Free Games API Service
//!
//! ## Rust concepts covered:
//! - Async HTTP requests with reqwest
//! - JSON deserialization with serde and nested structs
//! - Iterators: `into_iter()`, `filter_map()`, `flat_map()`, `find()`, `any()`
//! - `Option` chaining: `.as_ref()`, `.and_then()`, `.or_else()`, `.or()`
//! - `tokio::join!` for parallel async execution
//! - `Display` trait implementation
//! - `#[serde(rename_all)]` for JSON field name mapping

use chrono::{DateTime, Utc};
use serde::Deserialize;

/// A free game offer, unified across platforms.
#[derive(Debug, Clone)]
pub struct FreeGame {
    pub title: String,
    pub store: Store,
    pub url: String,
    pub original_price: Option<String>,
    pub end_date: Option<DateTime<Utc>>,
    pub image_url: Option<String>,
}

/// `PartialEq` and `Eq` enable `==` comparison.
/// `Copy` is valid here because the enum has no heap data — it's just a tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Store {
    Steam,
    EpicGames,
}

/// `Display` trait controls how a type is printed with `{}`.
/// Unlike `Debug` (which uses `{:?}`), `Display` is meant for user-facing output.
impl std::fmt::Display for Store {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Store::Steam => write!(f, "Steam"),
            Store::EpicGames => write!(f, "Epic Games"),
        }
    }
}

impl Store {
    pub fn emoji(&self) -> &'static str {
        match self {
            Store::Steam => "🎮",
            Store::EpicGames => "🎁",
        }
    }
}

// ============================================================================
// Epic Games Store API
// ============================================================================

/// Serde structs mirror the JSON structure from the API.
/// Fields not needed can be omitted — serde ignores unknown fields by default.
/// `#[serde(default)]` provides a fallback when the field is null or missing.
#[derive(Debug, Deserialize)]
struct EpicResponse {
    data: EpicData,
    #[serde(default)]
    #[allow(dead_code)]
    errors: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct EpicData {
    /// `#[serde(rename = "...")]` maps a JSON key to a different Rust field name
    #[serde(rename = "Catalog")]
    catalog: EpicCatalog,
}

/// `#[serde(rename_all = "camelCase")]` automatically converts
/// `search_store` (Rust) ↔ `searchStore` (JSON)
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EpicCatalog {
    search_store: EpicSearchStore,
}

#[derive(Debug, Deserialize)]
struct EpicSearchStore {
    elements: Vec<EpicGame>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EpicGame {
    title: String,
    #[serde(default)]
    key_images: Vec<EpicImage>,
    #[serde(default)]
    promotions: Option<EpicPromotions>,
    catalog_ns: EpicCatalogNs,
    url_slug: Option<String>,
    product_slug: Option<String>,
    /// `Option<Vec<T>>` handles both null and missing fields.
    /// The API returns null when it encounters partial errors.
    #[serde(default)]
    offer_mappings: Option<Vec<EpicOfferMapping>>,
}

#[derive(Debug, Deserialize)]
struct EpicCatalogNs {
    #[serde(default)]
    mappings: Option<Vec<EpicMapping>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EpicMapping {
    page_slug: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EpicOfferMapping {
    page_slug: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EpicImage {
    #[serde(rename = "type")]
    image_type: String,
    url: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EpicPromotions {
    #[serde(default)]
    promotional_offers: Vec<EpicPromoWrapper>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EpicPromoWrapper {
    promotional_offers: Vec<EpicPromoOffer>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EpicPromoOffer {
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    discount_setting: EpicDiscount,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EpicDiscount {
    discount_percentage: i32,
}

/// Fetches free games from the Epic Games Store API.
///
/// Demonstrates `filter_map` — combines filter and map in one step.
/// Return `Some(value)` to keep an element, `None` to skip it.
pub async fn fetch_epic_free_games() -> Result<Vec<FreeGame>, reqwest::Error> {
    let client = reqwest::Client::new();

    let response = client
        .get("https://store-site-backend-static-ipv4.ak.epicgames.com/freeGamesPromotions?locale=fr&country=FR&allowCountries=FR")
        .send()
        .await?
        // .json::<T>() deserializes the response body. The turbofish ::<T>
        // syntax tells the compiler what type to deserialize into.
        .json::<EpicResponse>()
        .await?;

    let now = Utc::now();

    // `into_iter()` takes ownership of the Vec (consuming it).
    // `filter_map` returns Some to keep, None to skip.
    let games: Vec<FreeGame> = response
        .data
        .catalog
        .search_store
        .elements
        .into_iter()
        .filter_map(|game| {
            // `?` on Option returns None early (like ? on Result returns Err)
            let promo = game.promotions.as_ref()?;

            // `flat_map` flattens nested iterators: Vec<Vec<T>> → Iterator<T>
            let current_offer = promo
                .promotional_offers
                .iter()
                .flat_map(|p| &p.promotional_offers)
                .find(|offer| {
                    offer.discount_setting.discount_percentage == 0
                        && offer.start_date <= now
                        && offer.end_date > now
                })?;

            // Option chaining: try each source for the URL slug, fall through on None.
            // .as_ref() borrows the Option's inner value without consuming it.
            // .and_then() maps Some(x) → f(x), passes through None.
            // .or_else() tries a fallback closure when the Option is None.
            // .or() tries a fallback Option value when the Option is None.
            let slug = game
                .catalog_ns
                .mappings
                .as_ref()
                .and_then(|m| m.first())
                .map(|m| m.page_slug.clone())
                .or_else(|| game.offer_mappings.as_ref().and_then(|m| m.first()).map(|m| m.page_slug.clone()))
                .or_else(|| game.product_slug.clone())
                .or(game.url_slug)?;

            let url = format!("https://store.epicgames.com/fr/p/{}", slug);

            let image_url = game
                .key_images
                .iter()
                .find(|img| img.image_type == "Thumbnail" || img.image_type == "OfferImageWide")
                .map(|img| img.url.clone());

            Some(FreeGame {
                title: game.title,
                store: Store::EpicGames,
                url,
                original_price: None,
                end_date: Some(current_offer.end_date),
                image_url,
            })
        })
        .collect(); // Collects the iterator into a Vec

    Ok(games)
}

// ============================================================================
// GamerPower API (Steam + other platforms)
// ============================================================================

#[derive(Debug, Deserialize)]
struct GamerPowerGame {
    title: String,
    worth: Option<String>,
    #[serde(rename = "open_giveaway_url")]
    open_giveaway_url: String,
    thumbnail: Option<String>,
    #[allow(dead_code)]
    platforms: String,
    end_date: Option<String>,
}

/// Fetches free PC games from the GamerPower API.
///
/// Demonstrates the builder pattern with `reqwest::Client::builder()`
/// and `.map()` / `.filter()` on iterators and Options.
pub async fn fetch_gamerpower_games() -> Result<Vec<FreeGame>, reqwest::Error> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let response = client
        .get("https://www.gamerpower.com/api/giveaways?platform=pc&type=game&sort-by=popularity")
        .send()
        .await;

    match response {
        Ok(resp) => {
            if let Ok(games) = resp.json::<Vec<GamerPowerGame>>().await {
                let free_games = games
                    .into_iter()
                    .map(|game| {
                        // .as_deref() converts Option<String> → Option<&str>
                        // .filter() keeps Some only if the predicate is true
                        let end_date = game.end_date
                            .as_deref()
                            .filter(|s| *s != "N/A")
                            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                            .map(|dt| dt.with_timezone(&Utc));

                        let original_price = game.worth
                            .filter(|w| w != "N/A");

                        // .contains() on &str checks for a substring
                        let store = if game.platforms.contains("Epic") {
                            Store::EpicGames
                        } else {
                            Store::Steam
                        };

                        // .replace() returns a new String — strings are immutable in Rust
                        let title = game.title
                            .replace(" (Epic Games) Giveaway", "")
                            .replace(" (Steam) Giveaway", "")
                            .replace(" Giveaway", "");

                        FreeGame {
                            title,
                            store,
                            url: game.open_giveaway_url,
                            original_price,
                            end_date,
                            image_url: game.thumbnail,
                        }
                    })
                    .collect();

                return Ok(free_games);
            }
        }
        Err(e) => {
            tracing::warn!("Failed to fetch GamerPower games: {}", e);
        }
    }

    Ok(Vec::new())
}

/// Fetches all free games from Epic API + GamerPower, with deduplication.
///
/// `tokio::join!` runs multiple futures concurrently on the same task —
/// both HTTP requests are in-flight simultaneously.
pub async fn fetch_all_free_games() -> Vec<FreeGame> {
    let (epic_result, gamerpower_result) = tokio::join!(
        fetch_epic_free_games(),
        fetch_gamerpower_games()
    );

    let mut all_games = Vec::new();

    if let Ok(games) = epic_result {
        // .extend() appends all elements from an iterator
        all_games.extend(games);
    } else {
        tracing::warn!("Failed to fetch Epic games");
    }

    // Deduplicate: skip GamerPower games already found via Epic API.
    // .any() returns true if any element matches the predicate (short-circuits).
    if let Ok(games) = gamerpower_result {
        for game in games {
            let title_lower = game.title.to_lowercase();
            let already_exists = all_games.iter().any(|existing| {
                existing.title.to_lowercase() == title_lower
            });
            if !already_exists {
                all_games.push(game);
            }
        }
    }

    // .sort_by() sorts in place using a custom comparator.
    // The closure receives references (&) to two elements and returns an Ordering.
    all_games.sort_by(|a, b| {
        match (&a.end_date, &b.end_date) {
            (Some(a_date), Some(b_date)) => a_date.cmp(b_date),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        }
    });

    all_games
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_display() {
        // .to_string() calls the Display trait implementation
        assert_eq!(Store::Steam.to_string(), "Steam");
        assert_eq!(Store::EpicGames.to_string(), "Epic Games");
    }

    #[test]
    fn test_store_emoji() {
        assert_eq!(Store::Steam.emoji(), "🎮");
        assert_eq!(Store::EpicGames.emoji(), "🎁");
    }
}
