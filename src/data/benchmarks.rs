//! Benchmark data aggregation
//!
//! Scrapes and caches optimized settings recommendations from:
//! - Digital Foundry (via search)
//! - Hardware Unboxed (via search)

use anyhow::{Context, Result};
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::cache::Cache;

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36";

/// Benchmark data client
pub struct BenchmarkClient {
    client: Client,
    cache: Cache,
}

impl BenchmarkClient {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        let cache = Cache::with_ttl(Duration::from_secs(7 * 24 * 60 * 60))?; // 1 week TTL

        Ok(Self { client, cache })
    }

    /// Search for benchmark/settings articles for a game
    pub async fn search(&self, game_name: &str) -> Result<Vec<BenchmarkArticle>> {
        let cache_key = format!("benchmarks_{}", game_name.to_lowercase().replace(' ', "_"));
        
        // Check cache first
        if let Some(cached) = self.cache.get::<Vec<BenchmarkArticle>>(&cache_key) {
            return Ok(cached);
        }

        let mut articles = Vec::new();

        // Search Digital Foundry
        if let Ok(df_articles) = self.search_digital_foundry(game_name).await {
            articles.extend(df_articles);
        }

        // Search Hardware Unboxed
        if let Ok(hub_articles) = self.search_hardware_unboxed(game_name).await {
            articles.extend(hub_articles);
        }

        // Cache results
        let _ = self.cache.set(&cache_key, &articles);

        Ok(articles)
    }

    /// Search Digital Foundry for optimized settings articles
    async fn search_digital_foundry(&self, game_name: &str) -> Result<Vec<BenchmarkArticle>> {
        let search_url = format!(
            "https://www.eurogamer.net/search?q={}+optimised+settings&type=article",
            urlencoding::encode(game_name)
        );

        let response = self.client
            .get(&search_url)
            .send()
            .await
            .context("Failed to search Digital Foundry")?;

        let html = response.text().await?;
        let document = Html::parse_document(&html);

        let mut articles = Vec::new();

        // Parse search results
        let article_selector = Selector::parse("article, .search-result, .article-item").unwrap();
        let title_selector = Selector::parse("h2 a, h3 a, .title a, a.article-link").unwrap();

        for article_el in document.select(&article_selector).take(5) {
            if let Some(link_el) = article_el.select(&title_selector).next() {
                let title = link_el.text().collect::<String>().trim().to_string();
                let url = link_el.value().attr("href").map(|s| {
                    if s.starts_with("http") {
                        s.to_string()
                    } else {
                        format!("https://www.eurogamer.net{}", s)
                    }
                });

                // Filter for relevant articles
                let title_lower = title.to_lowercase();
                if title_lower.contains("optimised") 
                    || title_lower.contains("optimized")
                    || title_lower.contains("settings")
                    || title_lower.contains("best settings")
                    || title_lower.contains("performance")
                {
                    articles.push(BenchmarkArticle {
                        title,
                        url,
                        source: BenchmarkSource::DigitalFoundry,
                        game: game_name.to_string(),
                        settings: None, // Would need to fetch and parse the article
                    });
                }
            }
        }

        Ok(articles)
    }

    /// Search Hardware Unboxed for settings guides
    async fn search_hardware_unboxed(&self, game_name: &str) -> Result<Vec<BenchmarkArticle>> {
        // Hardware Unboxed primarily uses YouTube, so we'll search their site
        let search_url = format!(
            "https://www.hardwareunboxed.com/?s={}+settings",
            urlencoding::encode(game_name)
        );

        let response = self.client
            .get(&search_url)
            .send()
            .await
            .context("Failed to search Hardware Unboxed")?;

        let html = response.text().await?;
        let document = Html::parse_document(&html);

        let mut articles = Vec::new();

        // Parse search results
        let article_selector = Selector::parse("article, .post, .entry").unwrap();
        let title_selector = Selector::parse("h2 a, .entry-title a, .post-title a").unwrap();

        for article_el in document.select(&article_selector).take(5) {
            if let Some(link_el) = article_el.select(&title_selector).next() {
                let title = link_el.text().collect::<String>().trim().to_string();
                let url = link_el.value().attr("href").map(String::from);

                let title_lower = title.to_lowercase();
                if title_lower.contains("settings")
                    || title_lower.contains("benchmark")
                    || title_lower.contains("performance")
                    || title_lower.contains("optimization")
                {
                    articles.push(BenchmarkArticle {
                        title,
                        url,
                        source: BenchmarkSource::HardwareUnboxed,
                        game: game_name.to_string(),
                        settings: None,
                    });
                }
            }
        }

        Ok(articles)
    }

    /// Fetch and parse settings from a specific article
    pub async fn fetch_article_settings(&self, article: &BenchmarkArticle) -> Result<Option<RecommendedSettings>> {
        let Some(url) = &article.url else {
            return Ok(None);
        };

        let cache_key = format!("article_{}", url.replace(['/', ':', '.'], "_"));
        
        if let Some(cached) = self.cache.get::<RecommendedSettings>(&cache_key) {
            return Ok(Some(cached));
        }

        let response = self.client.get(url).send().await?;
        let html = response.text().await?;
        let document = Html::parse_document(&html);

        // Try to extract settings from common patterns
        let settings = parse_settings_from_html(&document);

        if let Some(ref s) = settings {
            let _ = self.cache.set(&cache_key, s);
        }

        Ok(settings)
    }

    /// Clear the benchmark cache
    pub fn clear_cache(&self) -> Result<()> {
        self.cache.clear()
    }
}

impl Default for BenchmarkClient {
    fn default() -> Self {
        Self::new().expect("Failed to create BenchmarkClient")
    }
}

/// A benchmark/settings article
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkArticle {
    pub title: String,
    pub url: Option<String>,
    pub source: BenchmarkSource,
    pub game: String,
    pub settings: Option<RecommendedSettings>,
}

/// Source of benchmark data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BenchmarkSource {
    DigitalFoundry,
    HardwareUnboxed,
    Other,
}

impl std::fmt::Display for BenchmarkSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DigitalFoundry => write!(f, "Digital Foundry"),
            Self::HardwareUnboxed => write!(f, "Hardware Unboxed"),
            Self::Other => write!(f, "Other"),
        }
    }
}

/// Recommended settings extracted from an article
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecommendedSettings {
    /// Target resolution
    pub resolution: Option<String>,
    /// Target framerate
    pub target_fps: Option<String>,
    /// Graphics preset recommendation
    pub preset: Option<String>,
    /// Individual setting recommendations
    pub settings: Vec<SettingRecommendation>,
    /// General notes/observations
    pub notes: Vec<String>,
}

/// A single setting recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingRecommendation {
    pub name: String,
    pub value: String,
    pub impact: Option<PerformanceImpact>,
    pub notes: Option<String>,
}

/// Performance impact of a setting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PerformanceImpact {
    High,
    Medium,
    Low,
    Negligible,
}

impl std::fmt::Display for PerformanceImpact {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::High => write!(f, "High"),
            Self::Medium => write!(f, "Medium"),
            Self::Low => write!(f, "Low"),
            Self::Negligible => write!(f, "Negligible"),
        }
    }
}

/// Parse settings from article HTML (heuristic-based)
fn parse_settings_from_html(document: &Html) -> Option<RecommendedSettings> {
    let mut settings = RecommendedSettings::default();
    let mut found_any = false;

    // Look for common patterns in settings tables
    let table_selector = Selector::parse("table").ok()?;
    let row_selector = Selector::parse("tr").ok()?;
    let cell_selector = Selector::parse("td, th").ok()?;

    for table in document.select(&table_selector) {
        for row in table.select(&row_selector) {
            let cells: Vec<String> = row
                .select(&cell_selector)
                .map(|c| c.text().collect::<String>().trim().to_string())
                .collect();

            if cells.len() >= 2 {
                let name = &cells[0];
                let value = &cells[1];

                // Skip header rows
                if name.to_lowercase() == "setting" || name.to_lowercase() == "option" {
                    continue;
                }

                if !name.is_empty() && !value.is_empty() {
                    settings.settings.push(SettingRecommendation {
                        name: name.clone(),
                        value: value.clone(),
                        impact: infer_impact(name),
                        notes: cells.get(2).cloned(),
                    });
                    found_any = true;
                }
            }
        }
    }

    // Look for bullet points with settings
    let list_selector = Selector::parse("ul li, ol li").ok()?;
    for item in document.select(&list_selector) {
        let text = item.text().collect::<String>().trim().to_string();
        
        // Look for "Setting: Value" or "Setting - Value" patterns
        if let Some((name, value)) = parse_setting_line(&text) {
            settings.settings.push(SettingRecommendation {
                name,
                value,
                impact: None,
                notes: None,
            });
            found_any = true;
        }
    }

    // Extract notes from paragraphs mentioning performance
    let p_selector = Selector::parse("p").ok()?;
    for p in document.select(&p_selector).take(20) {
        let text = p.text().collect::<String>().trim().to_string();
        let text_lower = text.to_lowercase();
        
        if (text_lower.contains("recommend") || text_lower.contains("suggest"))
            && (text_lower.contains("fps") || text_lower.contains("performance"))
        {
            if text.len() < 500 {
                settings.notes.push(text);
            }
        }
    }

    if found_any || !settings.notes.is_empty() {
        Some(settings)
    } else {
        None
    }
}

/// Try to parse a "Setting: Value" line
fn parse_setting_line(text: &str) -> Option<(String, String)> {
    // Try colon separator
    if let Some(idx) = text.find(':') {
        let name = text[..idx].trim().to_string();
        let value = text[idx + 1..].trim().to_string();
        if !name.is_empty() && !value.is_empty() && name.len() < 50 {
            return Some((name, value));
        }
    }

    // Try dash separator
    if let Some(idx) = text.find(" - ") {
        let name = text[..idx].trim().to_string();
        let value = text[idx + 3..].trim().to_string();
        if !name.is_empty() && !value.is_empty() && name.len() < 50 {
            return Some((name, value));
        }
    }

    None
}

/// Infer performance impact from setting name
fn infer_impact(name: &str) -> Option<PerformanceImpact> {
    let name_lower = name.to_lowercase();
    
    // High impact settings
    if name_lower.contains("ray tracing")
        || name_lower.contains("raytracing")
        || name_lower.contains("rt ")
        || name_lower.contains("global illumination")
        || name_lower.contains("path tracing")
        || name_lower.contains("resolution")
        || name_lower.contains("render scale")
    {
        return Some(PerformanceImpact::High);
    }

    // Medium impact
    if name_lower.contains("shadow")
        || name_lower.contains("reflection")
        || name_lower.contains("ambient occlusion")
        || name_lower.contains("ssao")
        || name_lower.contains("volumetric")
        || name_lower.contains("particle")
        || name_lower.contains("draw distance")
        || name_lower.contains("view distance")
    {
        return Some(PerformanceImpact::Medium);
    }

    // Low impact
    if name_lower.contains("texture")
        || name_lower.contains("anisotropic")
        || name_lower.contains("anti-alias")
        || name_lower.contains("antialiasing")
        || name_lower.contains("motion blur")
        || name_lower.contains("depth of field")
        || name_lower.contains("bloom")
    {
        return Some(PerformanceImpact::Low);
    }

    None
}
