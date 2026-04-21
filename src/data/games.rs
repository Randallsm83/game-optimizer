//! Unified game detection across all platforms
//!
//! Aggregates games from Steam, Xbox, GOG, Epic, and EA

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::steam::SteamLibrary;
use super::xbox::XboxLibrary;
use super::gog::GogLibrary;
use super::epic::EpicLibrary;
use super::ea::EaLibrary;

/// Platform/store a game is from
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GamePlatform {
    Steam,
    Xbox,
    Gog,
    Epic,
    Ea,
}

impl std::fmt::Display for GamePlatform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GamePlatform::Steam => write!(f, "Steam"),
            GamePlatform::Xbox => write!(f, "Xbox/MS Store"),
            GamePlatform::Gog => write!(f, "GOG"),
            GamePlatform::Epic => write!(f, "Epic"),
            GamePlatform::Ea => write!(f, "EA App"),
        }
    }
}

/// A game detected from any platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedGame {
    pub name: String,
    pub platform: GamePlatform,
    pub install_path: Option<PathBuf>,
    /// Platform-specific identifier (Steam AppID, Xbox PackageFamilyName, etc.)
    pub platform_id: String,
}

/// Scan all platforms for installed games
pub fn detect_all_games() -> Result<Vec<DetectedGame>> {
    let mut all_games = Vec::new();

    // Steam
    if let Ok(steam) = SteamLibrary::new() {
        if let Ok(games) = steam.installed_games() {
            for game in games {
                all_games.push(DetectedGame {
                    name: game.name.clone(),
                    platform: GamePlatform::Steam,
                    install_path: Some(game.install_dir),
                    platform_id: game.appid.to_string(),
                });
            }
        }
    }

    // Xbox/Microsoft Store
    if let Ok(games) = XboxLibrary::installed_games() {
        for game in games {
            all_games.push(DetectedGame {
                name: game.name,
                platform: GamePlatform::Xbox,
                install_path: game.install_location,
                platform_id: game.package_family_name,
            });
        }
    }

    // GOG
    if let Ok(gog) = GogLibrary::new() {
        if let Ok(games) = gog.installed_games() {
            for game in games {
                all_games.push(DetectedGame {
                    name: game.name,
                    platform: GamePlatform::Gog,
                    install_path: game.install_path,
                    platform_id: game.product_id.to_string(),
                });
            }
        }
    }

    // Epic
    if let Ok(epic) = EpicLibrary::new() {
        if let Ok(games) = epic.installed_games() {
            for game in games {
                all_games.push(DetectedGame {
                    name: game.name,
                    platform: GamePlatform::Epic,
                    install_path: Some(game.install_path),
                    platform_id: game.app_name,
                });
            }
        }
    }

    // EA App
    if let Ok(ea) = EaLibrary::new() {
        if let Ok(games) = ea.installed_games() {
            for game in games {
                all_games.push(DetectedGame {
                    name: game.name,
                    platform: GamePlatform::Ea,
                    install_path: Some(game.install_path),
                    platform_id: game.content_id,
                });
            }
        }
    }

    // Sort alphabetically
    all_games.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    Ok(all_games)
}

/// Find games by name across all platforms
pub fn find_games_by_name(name: &str) -> Result<Vec<DetectedGame>> {
    let name_lower = name.to_lowercase();
    let all = detect_all_games()?;
    Ok(all
        .into_iter()
        .filter(|g| g.name.to_lowercase().contains(&name_lower))
        .collect())
}

/// Detect games from a specific platform only
pub fn detect_games_from(platform: GamePlatform) -> Result<Vec<DetectedGame>> {
    let all = detect_all_games()?;
    Ok(all.into_iter().filter(|g| g.platform == platform).collect())
}
