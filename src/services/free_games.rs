//! # Service de récupération des jeux gratuits
//!
//! ## Concepts Rust illustrés :
//! - HTTP requests asynchrones avec reqwest
//! - Désérialisation JSON avec serde
//! - Error handling avec Result
//! - Structs imbriquées pour mapper des APIs

use chrono::{DateTime, Utc};
use serde::Deserialize;

/// Représente un jeu gratuit (unifié entre les plateformes).
///
/// ## Concept Rust : Debug et Clone
/// - `Debug` permet d'afficher avec `{:?}` pour le debugging
/// - `Clone` permet de dupliquer la struct
#[derive(Debug, Clone)]
pub struct FreeGame {
    pub title: String,
    pub store: Store,
    pub url: String,
    pub original_price: Option<String>,
    pub end_date: Option<DateTime<Utc>>,
    pub image_url: Option<String>,
}

/// Plateforme de jeu.
///
/// ## Concept Rust : Enum simple avec Display
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Store {
    Steam,
    EpicGames,
}

impl std::fmt::Display for Store {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Store::Steam => write!(f, "Steam"),
            Store::EpicGames => write!(f, "Epic Games"),
        }
    }
}

impl Store {
    /// Retourne l'emoji correspondant à la plateforme.
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

/// Structure pour désérialiser la réponse de l'API Epic Games.
///
/// ## Concept Rust : Serde attributes
/// `#[serde(rename_all = "camelCase")]` convertit automatiquement
/// les noms de champs de camelCase (JSON) vers snake_case (Rust).
#[derive(Debug, Deserialize)]
struct EpicResponse {
    data: EpicData,
}

#[derive(Debug, Deserialize)]
struct EpicData {
    #[serde(rename = "Catalog")]
    catalog: EpicCatalog,
}

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
    #[serde(default)]
    offer_mappings: Vec<EpicOfferMapping>,
}

#[derive(Debug, Deserialize)]
struct EpicCatalogNs {
    mappings: Vec<EpicMapping>,
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

/// Récupère les jeux gratuits depuis Epic Games Store.
///
/// ## Concept Rust : async fn et Result
/// La fonction est async car elle fait des I/O réseau.
/// Elle retourne un Result pour gérer les erreurs proprement.
pub async fn fetch_epic_free_games() -> Result<Vec<FreeGame>, reqwest::Error> {
    let client = reqwest::Client::new();

    // L'API GraphQL d'Epic Games pour les jeux gratuits
    // ## Concept : Raw string literals r#"..."#
    // Permet d'inclure des guillemets sans les échapper
    // Note: On utilise l'API REST ci-dessous, mais voici la query GraphQL pour référence
    let _query = r#"
    query {
        Catalog {
            searchStore(
                category: "games/edition/base"
                count: 20
                effectiveDate: "[,]"
                freeGame: true
                onSale: true
                sortBy: "effectiveDate"
                sortDir: "asc"
            ) {
                elements {
                    title
                    keyImages {
                        type
                        url
                    }
                    promotions {
                        promotionalOffers {
                            promotionalOffers {
                                startDate
                                endDate
                                discountSetting {
                                    discountPercentage
                                }
                            }
                        }
                    }
                    catalogNs {
                        mappings {
                            pageSlug
                        }
                    }
                    urlSlug
                    offerMappings {
                        pageSlug
                    }
                }
            }
        }
    }
    "#;

    let response = client
        .get("https://store-site-backend-static-ipv4.ak.epicgames.com/freeGamesPromotions?locale=fr&country=FR&allowCountries=FR")
        .send()
        .await?
        .json::<EpicResponse>()
        .await?;

    let now = Utc::now();

    // ## Concept : Iterator et filter_map
    // filter_map combine filter et map : retourne Some pour garder, None pour ignorer
    let games: Vec<FreeGame> = response
        .data
        .catalog
        .search_store
        .elements
        .into_iter()
        .filter_map(|game| {
            // Vérifier que c'est actuellement gratuit (100% de réduction)
            let promo = game.promotions.as_ref()?;
            let current_offer = promo
                .promotional_offers
                .iter()
                .flat_map(|p| &p.promotional_offers)
                .find(|offer| {
                    offer.discount_setting.discount_percentage == 0
                        && offer.start_date <= now
                        && offer.end_date > now
                })?;

            // Construire l'URL
            let slug = game
                .catalog_ns
                .mappings
                .first()
                .map(|m| m.page_slug.clone())
                .or_else(|| game.offer_mappings.first().map(|m| m.page_slug.clone()))
                .or(game.url_slug)?;

            let url = format!("https://store.epicgames.com/fr/p/{}", slug);

            // Trouver l'image
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
        .collect();

    Ok(games)
}

// ============================================================================
// Steam Free Games (via web scraping d'une API communautaire)
// ============================================================================

/// Structure pour l'API Steam des apps gratuites.
#[derive(Debug, Deserialize)]
struct SteamFreeGame {
    title: String,
    #[serde(rename = "app_id")]
    app_id: u64,
    original_price: Option<String>,
    end_date: Option<i64>,
}

/// Récupère les jeux temporairement gratuits sur Steam.
///
/// ## Concept Rust : Fallback et error handling
/// On utilise une API communautaire, avec fallback si elle échoue.
pub async fn fetch_steam_free_games() -> Result<Vec<FreeGame>, reqwest::Error> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    // API communautaire pour les free games Steam
    // Note: Steam n'a pas d'API officielle facile pour ça
    let response = client
        .get("https://steam-free-games-api.vercel.app/api/free-games")
        .send()
        .await;

    // ## Concept : match avec Result
    match response {
        Ok(resp) => {
            if let Ok(games) = resp.json::<Vec<SteamFreeGame>>().await {
                let free_games = games
                    .into_iter()
                    .map(|game| {
                        let url = format!(
                            "https://store.steampowered.com/app/{}",
                            game.app_id
                        );
                        let end_date = game.end_date.map(|ts| {
                            DateTime::<Utc>::from_timestamp(ts, 0)
                                .unwrap_or_else(Utc::now)
                        });

                        FreeGame {
                            title: game.title,
                            store: Store::Steam,
                            url,
                            original_price: game.original_price,
                            end_date,
                            image_url: Some(format!(
                                "https://cdn.cloudflare.steamstatic.com/steam/apps/{}/header.jpg",
                                game.app_id
                            )),
                        }
                    })
                    .collect();

                return Ok(free_games);
            }
        }
        Err(e) => {
            tracing::warn!("Impossible de récupérer les jeux Steam gratuits: {}", e);
        }
    }

    // Fallback: retourner une liste vide
    Ok(Vec::new())
}

/// Récupère tous les jeux gratuits (Steam + Epic).
///
/// ## Concept Rust : tokio::join! pour parallélisme
/// Exécute les deux requêtes en parallèle plutôt que séquentiellement.
pub async fn fetch_all_free_games() -> Vec<FreeGame> {
    // ## Concept : Parallel async avec tokio::join!
    let (epic_result, steam_result) = tokio::join!(
        fetch_epic_free_games(),
        fetch_steam_free_games()
    );

    let mut all_games = Vec::new();

    // ## Concept : if let Ok pour ignorer les erreurs
    if let Ok(games) = epic_result {
        all_games.extend(games);
    } else {
        tracing::warn!("Erreur lors de la récupération des jeux Epic");
    }

    if let Ok(games) = steam_result {
        all_games.extend(games);
    }

    // Trier par date de fin (les plus urgents en premier)
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
        assert_eq!(Store::Steam.to_string(), "Steam");
        assert_eq!(Store::EpicGames.to_string(), "Epic Games");
    }

    #[test]
    fn test_store_emoji() {
        assert_eq!(Store::Steam.emoji(), "🎮");
        assert_eq!(Store::EpicGames.emoji(), "🎁");
    }
}
