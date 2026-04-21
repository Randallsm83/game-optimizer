//! Xbox/Microsoft Store game detection
//!
//! Detects games installed via Xbox App / Game Pass / Microsoft Store

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

/// An Xbox/Microsoft Store game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XboxGame {
    pub name: String,
    pub package_family_name: String,
    pub publisher: Option<String>,
    pub version: Option<String>,
    pub install_location: Option<PathBuf>,
}

/// Xbox game library manager
pub struct XboxLibrary;

impl XboxLibrary {
    /// Get all installed Xbox/Microsoft Store games
    /// Uses PowerShell to query AppxPackages
    pub fn installed_games() -> Result<Vec<XboxGame>> {
        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                r#"Get-AppxPackage | Where-Object { $_.IsFramework -eq $false -and $_.SignatureKind -eq 'Store' } | Select-Object Name, PackageFamilyName, Publisher, Version, InstallLocation | ConvertTo-Json -Compress"#,
            ])
            .output()
            .context("Failed to run PowerShell")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("PowerShell command failed: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.trim().is_empty() {
            return Ok(Vec::new());
        }

        // Parse JSON output
        let packages: Vec<AppxPackage> = serde_json::from_str(&stdout)
            .or_else(|_| {
                // Single result comes as object, not array
                serde_json::from_str::<AppxPackage>(&stdout).map(|p| vec![p])
            })
            .unwrap_or_default();

        // Filter to likely games (heuristic: has "game" in name or known game publishers)
        let games: Vec<XboxGame> = packages
            .into_iter()
            .filter(|p| is_likely_game(p))
            .map(|p| XboxGame {
                name: clean_package_name(&p.name),
                package_family_name: p.package_family_name,
                publisher: p.publisher,
                version: p.version,
                install_location: p.install_location.map(PathBuf::from),
            })
            .collect();

        Ok(games)
    }

    /// Find a game by name (case-insensitive partial match)
    pub fn find_by_name(name: &str) -> Result<Vec<XboxGame>> {
        let name_lower = name.to_lowercase();
        let games = Self::installed_games()?;
        Ok(games
            .into_iter()
            .filter(|g| g.name.to_lowercase().contains(&name_lower))
            .collect())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AppxPackage {
    name: String,
    package_family_name: String,
    publisher: Option<String>,
    version: Option<String>,
    install_location: Option<String>,
}

/// Heuristic to filter apps to likely games
fn is_likely_game(package: &AppxPackage) -> bool {
    let name_lower = package.name.to_lowercase();

    // Exclude known non-games first
    let exclude_patterns = [
        "microsoft.",
        "microsoftcorporationii.",
        "microsoftwindows.",
        "windows.",
        ".net",
        "nvidiacorp",
        "intel",
        "realtek",
        "cortana",
        "xbox identity",
        "xbox tcui",
        "xbox speech",
        "gamingservices",
        "your phone",
        "photos",
        "calculator",
        "camera",
        "mail",
        "maps",
        "weather",
        "news",
        "office",
        "onenote",
        "onedrive",
        "edge",
        "store purchase",
        "clipchamp",
        "dolby",
        "speech.",
        "crossdevice",
        "webexperience",
        // Command palette extensions
        "commandpalet",
        "scoopextension",
        "everythingcp",
        "wintoys",
        "steampal",
    ];

    if exclude_patterns.iter().any(|p| name_lower.contains(p)) {
        return false;
    }

    // If it survived the exclusion list, it's likely a game
    // (most non-game Store apps are from Microsoft)
    true
}

/// Clean up package name for display
fn clean_package_name(name: &str) -> String {
    // Strip publisher prefix (e.g., "HelloGames.NoMansSky" -> "NoMansSky")
    let game_name = name.split('.').last().unwrap_or(name);

    // Convert CamelCase to spaces
    let mut result = String::new();
    for (i, c) in game_name.chars().enumerate() {
        if i > 0 && c.is_uppercase() {
            result.push(' ');
        }
        result.push(c);
    }

    result
}
