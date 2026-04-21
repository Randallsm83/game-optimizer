//! Epic Games Launcher game detection
//!
//! Reads installed games from Epic's manifest files

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// An Epic Games game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpicGame {
    pub name: String,
    pub app_name: String,
    pub install_path: PathBuf,
    pub version: Option<String>,
    pub size_bytes: Option<u64>,
}

/// Epic Games library manager
pub struct EpicLibrary {
    manifests_path: PathBuf,
}

impl EpicLibrary {
    /// Create a new Epic library manager with auto-detected path
    pub fn new() -> Result<Self> {
        let manifests_path = detect_epic_manifests_path()?;
        Ok(Self { manifests_path })
    }

    /// Get all installed Epic games
    pub fn installed_games(&self) -> Result<Vec<EpicGame>> {
        let mut games = Vec::new();

        if !self.manifests_path.exists() {
            return Ok(games);
        }

        if let Ok(entries) = std::fs::read_dir(&self.manifests_path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.extension().map(|e| e == "item").unwrap_or(false) {
                if let Ok(manifest) = parse_epic_manifest(&path) {
                        if !manifest.install_location.is_empty() {
                            games.push(EpicGame {
                                name: manifest.display_name,
                                app_name: manifest.app_name,
                                install_path: PathBuf::from(&manifest.install_location),
                                version: Some(manifest.app_version),
                                size_bytes: Some(manifest.install_size),
                            });
                        }
                    }
                }
            }
        }

        games.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        Ok(games)
    }

    /// Find a game by name
    pub fn find_by_name(&self, name: &str) -> Result<Vec<EpicGame>> {
        let name_lower = name.to_lowercase();
        let games = self.installed_games()?;
        Ok(games
            .into_iter()
            .filter(|g| g.name.to_lowercase().contains(&name_lower))
            .collect())
    }
}

impl EpicGame {
    /// Format size as human-readable
    pub fn size_display(&self) -> String {
        match self.size_bytes {
            Some(bytes) => {
                const GB: u64 = 1024 * 1024 * 1024;
                const MB: u64 = 1024 * 1024;
                if bytes >= GB {
                    format!("{:.1} GB", bytes as f64 / GB as f64)
                } else {
                    format!("{:.0} MB", bytes as f64 / MB as f64)
                }
            }
            None => "Unknown".to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct EpicManifest {
    display_name: String,
    app_name: String,
    install_location: String,
    #[serde(default)]
    app_version: String,
    #[serde(rename = "InstallSize", default)]
    install_size: u64,
}

fn parse_epic_manifest(path: &PathBuf) -> Result<EpicManifest> {
    let content = std::fs::read_to_string(path)
        .context("Failed to read Epic manifest")?;
    let manifest: EpicManifest = serde_json::from_str(&content)
        .context("Failed to parse Epic manifest")?;
    Ok(manifest)
}

/// Detect Epic Games manifests path
fn detect_epic_manifests_path() -> Result<PathBuf> {
    // Default location: %ProgramData%\Epic\EpicGamesLauncher\Data\Manifests
    let program_data = std::env::var("ProgramData")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(r"C:\ProgramData"));

    let manifests_path = program_data
        .join("Epic")
        .join("EpicGamesLauncher")
        .join("Data")
        .join("Manifests");

    Ok(manifests_path)
}
