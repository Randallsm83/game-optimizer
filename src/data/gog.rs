//! GOG Galaxy game detection
//!
//! Reads installed games from GOG Galaxy's SQLite database

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A GOG Galaxy game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GogGame {
    pub name: String,
    pub product_id: u64,
    pub install_path: Option<PathBuf>,
}

/// GOG Galaxy library manager
pub struct GogLibrary {
    db_path: PathBuf,
}

impl GogLibrary {
    /// Create a new GOG library manager with auto-detected path
    pub fn new() -> Result<Self> {
        let db_path = detect_gog_db_path()?;
        Ok(Self { db_path })
    }

    /// Get the database path
    pub fn db_path(&self) -> &PathBuf {
        &self.db_path
    }

    /// Get all installed GOG games
    /// Note: Requires rusqlite feature to be enabled for full functionality
    pub fn installed_games(&self) -> Result<Vec<GogGame>> {
        // For now, we'll parse the database using a simple approach
        // A full implementation would use rusqlite
        
        if !self.db_path.exists() {
            return Ok(Vec::new());
        }

        // Try to read installed games from the config file instead
        // GOG also stores info in %LOCALAPPDATA%\GOG.com\Galaxy\Configuration\config.json
        let config_path = dirs::data_local_dir()
            .unwrap_or_default()
            .join("GOG.com")
            .join("Galaxy")
            .join("Configuration")
            .join("config.json");

        if config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                if let Ok(config) = serde_json::from_str::<GogConfig>(&content) {
                    if let Some(library_path) = config.library_path {
                        return scan_gog_library(&PathBuf::from(library_path));
                    }
                }
            }
        }

        // Fallback: scan default GOG Games folder
        let default_path = PathBuf::from(r"C:\GOG Games");
        if default_path.exists() {
            return scan_gog_library(&default_path);
        }

        Ok(Vec::new())
    }

    /// Find a game by name
    pub fn find_by_name(&self, name: &str) -> Result<Vec<GogGame>> {
        let name_lower = name.to_lowercase();
        let games = self.installed_games()?;
        Ok(games
            .into_iter()
            .filter(|g| g.name.to_lowercase().contains(&name_lower))
            .collect())
    }
}

#[derive(Debug, Deserialize)]
struct GogConfig {
    #[serde(rename = "libraryPath")]
    library_path: Option<String>,
}

/// Scan a GOG library folder for installed games
fn scan_gog_library(path: &PathBuf) -> Result<Vec<GogGame>> {
    let mut games = Vec::new();

    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.filter_map(|e| e.ok()) {
            let game_path = entry.path();
            if game_path.is_dir() {
                // Look for goggame-*.info file
                if let Ok(info_files) = std::fs::read_dir(&game_path) {
                    for info_entry in info_files.filter_map(|e| e.ok()) {
                        let info_path = info_entry.path();
                        if let Some(name) = info_path.file_name().and_then(|n| n.to_str()) {
                            if name.starts_with("goggame-") && name.ends_with(".info") {
                                if let Ok(info) = parse_gog_info(&info_path) {
                                    games.push(GogGame {
                                        name: info.name,
                                        product_id: info.game_id.parse().unwrap_or(0),
                                        install_path: Some(game_path.clone()),
                                    });
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    games.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(games)
}

#[derive(Debug, Deserialize)]
struct GogGameInfo {
    #[serde(rename = "gameId")]
    game_id: String,
    name: String,
}

fn parse_gog_info(path: &PathBuf) -> Result<GogGameInfo> {
    let content = std::fs::read_to_string(path)?;
    let info: GogGameInfo = serde_json::from_str(&content)?;
    Ok(info)
}

/// Detect GOG Galaxy database path
fn detect_gog_db_path() -> Result<PathBuf> {
    // Default location: %ProgramData%\GOG.com\Galaxy\storage\galaxy-2.0.db
    let program_data = std::env::var("ProgramData")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(r"C:\ProgramData"));

    let db_path = program_data
        .join("GOG.com")
        .join("Galaxy")
        .join("storage")
        .join("galaxy-2.0.db");

    Ok(db_path)
}
