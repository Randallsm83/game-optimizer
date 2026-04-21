//! PCGamingWiki API client
//!
//! Uses the MediaWiki Cargo API to query game information.
//! API docs: https://www.pcgamingwiki.com/wiki/PCGamingWiki:API

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

const PCGW_API_URL: &str = "https://www.pcgamingwiki.com/w/api.php";

/// PCGamingWiki API client
pub struct PcgwClient {
    client: Client,
}

impl PcgwClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// Query game info by Steam AppID
    pub async fn get_by_steam_appid(&self, appid: u32) -> Result<Option<GameInfo>> {
        // First, get basic game info from Infobox_game table
        let infobox_url = format!(
            "{}?action=cargoquery&tables=Infobox_game&fields=\
            Infobox_game._pageName=Page,\
            Infobox_game.Developers,\
            Infobox_game.Publishers,\
            Infobox_game.Engines,\
            Infobox_game.Released,\
            Infobox_game.Cover_URL\
            &where=Infobox_game.Steam_AppID%20HOLDS%20%22{}%22\
            &format=json",
            PCGW_API_URL, appid
        );

        let response: CargoQueryResponse = self.client
            .get(&infobox_url)
            .send()
            .await
            .context("Failed to query PCGamingWiki")?
            .json()
            .await
            .context("Failed to parse PCGamingWiki response")?;

        let Some(first) = response.cargoquery.first() else {
            return Ok(None);
        };

        let page_name = get_string(&first.title, "Page").unwrap_or_default();
        if page_name.is_empty() {
            return Ok(None);
        }

        // Now get video settings from Video table
        let video_info = self.get_video_settings(appid).await.ok().flatten();

        // Get input settings
        let input_info = self.get_input_settings(appid).await.ok().flatten();

        Ok(Some(GameInfo {
            name: page_name,
            steam_appid: Some(appid),
            developers: parse_list_json(first.title.get("Developers")),
            publishers: parse_list_json(first.title.get("Publishers")),
            engines: parse_list_json(first.title.get("Engines")),
            release_date: get_string(&first.title, "Released"),
            cover_url: get_string(&first.title, "Cover URL"),
            video: video_info,
            input: input_info,
        }))
    }

    /// Query game info by name
    pub async fn get_by_name(&self, name: &str) -> Result<Option<GameInfo>> {
        let encoded_name = urlencoding::encode(name);
        let url = format!(
            "{}?action=cargoquery&tables=Infobox_game&fields=\
            Infobox_game._pageName=Page,\
            Infobox_game.Steam_AppID,\
            Infobox_game.Developers,\
            Infobox_game.Publishers,\
            Infobox_game.Engines,\
            Infobox_game.Released\
            &where=Infobox_game._pageName%20LIKE%20%22%25{}%25%22\
            &limit=1&format=json",
            PCGW_API_URL, encoded_name
        );

        let response: CargoQueryResponse = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to query PCGamingWiki")?
            .json()
            .await
            .context("Failed to parse PCGamingWiki response")?;

        let Some(first) = response.cargoquery.first() else {
            return Ok(None);
        };

        let page_name = get_string(&first.title, "Page").unwrap_or_default();
        let steam_appid = get_string(&first.title, "Steam AppID")
            .and_then(|s| s.parse::<u32>().ok());

        // Get additional details if we have a Steam AppID
        let (video_info, input_info) = if let Some(appid) = steam_appid {
            (
                self.get_video_settings(appid).await.ok().flatten(),
                self.get_input_settings(appid).await.ok().flatten(),
            )
        } else {
            (None, None)
        };

        Ok(Some(GameInfo {
            name: page_name,
            steam_appid,
            developers: parse_list_json(first.title.get("Developers")),
            publishers: parse_list_json(first.title.get("Publishers")),
            engines: parse_list_json(first.title.get("Engines")),
            release_date: get_string(&first.title, "Released"),
            cover_url: None,
            video: video_info,
            input: input_info,
        }))
    }

    /// Get video/display settings for a game
    async fn get_video_settings(&self, appid: u32) -> Result<Option<VideoSettings>> {
        let url = format!(
            "{}?action=cargoquery&tables=Infobox_game,Video\
            &fields=\
            Video.Widescreen_resolution,\
            Video.Ultrawidescreen,\
            Video.Multimonitor,\
            Video.4K_Ultra_HD,\
            Video.HDR,\
            Video.Ray_tracing,\
            Video.Vsync,\
            Video.60_FPS,\
            Video.120_FPS,\
            Video.Upscaling,\
            Video.Frame_gen\
            &join_on=Infobox_game._pageID=Video._pageID\
            &where=Infobox_game.Steam_AppID%20HOLDS%20%22{}%22\
            &format=json",
            PCGW_API_URL, appid
        );

        let response: CargoQueryResponse = self.client
            .get(&url)
            .send()
            .await?
            .json()
            .await?;

        let Some(first) = response.cargoquery.first() else {
            return Ok(None);
        };

        Ok(Some(VideoSettings {
            widescreen: parse_support_json(first.title.get("Widescreen resolution")),
            ultrawidescreen: parse_support_json(first.title.get("Ultrawidescreen")),
            super_ultrawide: SupportLevel::Unknown, // Not in current schema
            multi_monitor: parse_support_json(first.title.get("Multimonitor")),
            four_k: parse_support_json(first.title.get("4K Ultra HD")),
            hdr: parse_support_json(first.title.get("HDR")),
            ray_tracing: parse_support_json(first.title.get("Ray tracing")),
            vsync: parse_support_json(first.title.get("Vsync")),
            fps_60: parse_support_json(first.title.get("60 FPS")),
            fps_120: parse_support_json(first.title.get("120 FPS")),
            fps_unlimited: SupportLevel::Unknown, // Not in current schema
        }))
    }

    /// Get input settings for a game
    async fn get_input_settings(&self, appid: u32) -> Result<Option<InputSettings>> {
        let url = format!(
            "{}?action=cargoquery&tables=Infobox_game,Input\
            &fields=\
            Input.Full_controller,\
            Input.Controller_remapping,\
            Input.Controller_sensitivity,\
            Input.Controller_Y_axis_inversion\
            &join_on=Infobox_game._pageID=Input._pageID\
            &where=Infobox_game.Steam_AppID%20HOLDS%20%22{}%22\
            &format=json",
            PCGW_API_URL, appid
        );

        let response: CargoQueryResponse = self.client
            .get(&url)
            .send()
            .await?
            .json()
            .await?;

        let Some(first) = response.cargoquery.first() else {
            return Ok(None);
        };

        Ok(Some(InputSettings {
            full_controller: parse_support_json(first.title.get("Full controller")),
            controller_remapping: parse_support_json(first.title.get("Controller remapping")),
            controller_sensitivity: parse_support_json(first.title.get("Controller sensitivity")),
            controller_y_inversion: parse_support_json(first.title.get("Controller Y-axis inversion")),
        }))
    }
}

impl Default for PcgwClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Cargo API response wrapper
#[derive(Debug, Deserialize)]
struct CargoQueryResponse {
    cargoquery: Vec<CargoQueryResult>,
}

#[derive(Debug, Deserialize)]
struct CargoQueryResult {
    title: std::collections::HashMap<String, serde_json::Value>,
}

/// Game information from PCGamingWiki
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameInfo {
    pub name: String,
    pub steam_appid: Option<u32>,
    pub developers: Vec<String>,
    pub publishers: Vec<String>,
    pub engines: Vec<String>,
    pub release_date: Option<String>,
    pub cover_url: Option<String>,
    pub video: Option<VideoSettings>,
    pub input: Option<InputSettings>,
}

/// Video/display settings support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoSettings {
    pub widescreen: SupportLevel,
    pub ultrawidescreen: SupportLevel,
    pub super_ultrawide: SupportLevel,
    pub multi_monitor: SupportLevel,
    pub four_k: SupportLevel,
    pub hdr: SupportLevel,
    pub ray_tracing: SupportLevel,
    pub vsync: SupportLevel,
    pub fps_60: SupportLevel,
    pub fps_120: SupportLevel,
    pub fps_unlimited: SupportLevel,
}

/// Input settings support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputSettings {
    pub full_controller: SupportLevel,
    pub controller_remapping: SupportLevel,
    pub controller_sensitivity: SupportLevel,
    pub controller_y_inversion: SupportLevel,
}

/// Support level for a feature
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SupportLevel {
    Native,      // true
    Hackable,    // hackable
    Limited,     // limited
    Unsupported, // false
    Unknown,     // empty/null
}

impl std::fmt::Display for SupportLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Native => write!(f, "✓ Native"),
            Self::Hackable => write!(f, "⚙ Hackable"),
            Self::Limited => write!(f, "~ Limited"),
            Self::Unsupported => write!(f, "✗ No"),
            Self::Unknown => write!(f, "? Unknown"),
        }
    }
}

/// Get a string value from JSON, handling null
fn get_string(map: &std::collections::HashMap<String, serde_json::Value>, key: &str) -> Option<String> {
    map.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn parse_support_json(value: Option<&serde_json::Value>) -> SupportLevel {
    match value.and_then(|v| v.as_str()).map(|s| s.to_lowercase()).as_deref() {
        Some("true") => SupportLevel::Native,
        Some("hackable") => SupportLevel::Hackable,
        Some("limited") => SupportLevel::Limited,
        Some("false") => SupportLevel::Unsupported,
        _ => SupportLevel::Unknown,
    }
}

fn parse_list_json(value: Option<&serde_json::Value>) -> Vec<String> {
    value
        .and_then(|v| v.as_str())
        .map(|s| s.split(',').map(|p| p.trim().to_string()).filter(|s| !s.is_empty()).collect())
        .unwrap_or_default()
}
