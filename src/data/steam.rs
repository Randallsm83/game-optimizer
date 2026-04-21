//! Steam library detection
//!
//! Finds installed Steam games by parsing libraryfolders.vdf

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// A Steam game installation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteamGame {
    pub appid: u32,
    pub name: String,
    pub install_dir: PathBuf,
    pub size_bytes: Option<u64>,
    pub launch_options: Option<String>,
}

/// Steam library manager
pub struct SteamLibrary {
    steam_path: PathBuf,
}

impl SteamLibrary {
    /// Create a new Steam library manager with auto-detected path
    pub fn new() -> Result<Self> {
        let steam_path = detect_steam_path()?;
        Ok(Self { steam_path })
    }

    /// Create with a specific Steam path
    pub fn with_path(steam_path: PathBuf) -> Self {
        Self { steam_path }
    }

    /// Get the Steam installation path
    pub fn steam_path(&self) -> &PathBuf {
        &self.steam_path
    }

    /// Get all library folders (including the main Steam folder)
    pub fn library_folders(&self) -> Result<Vec<PathBuf>> {
        let vdf_path = self.steam_path.join("steamapps").join("libraryfolders.vdf");
        
        if !vdf_path.exists() {
            // Return just the main steamapps folder
            return Ok(vec![self.steam_path.join("steamapps")]);
        }

        let content = std::fs::read_to_string(&vdf_path)
            .context("Failed to read libraryfolders.vdf")?;

        let mut folders = vec![self.steam_path.join("steamapps")];

        // Parse VDF format (simplified parser)
        // Looking for "path" entries
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("\"path\"") {
                if let Some(path) = extract_vdf_value(line) {
                    let library_path = PathBuf::from(path).join("steamapps");
                    if library_path.exists() && !folders.contains(&library_path) {
                        folders.push(library_path);
                    }
                }
            }
        }

        Ok(folders)
    }

    /// Get all installed games across all library folders
    pub fn installed_games(&self) -> Result<Vec<SteamGame>> {
        let folders = self.library_folders()?;
        let mut games = Vec::new();

        for folder in folders {
            if let Ok(entries) = std::fs::read_dir(&folder) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.extension().map(|e| e == "acf").unwrap_or(false) {
                        if let Ok(game) = parse_app_manifest(&path, &folder) {
                            games.push(game);
                        }
                    }
                }
            }
        }

        // Sort by name
        games.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        Ok(games)
    }

    /// Find a game by AppID
    pub fn find_by_appid(&self, appid: u32) -> Result<Option<SteamGame>> {
        let games = self.installed_games()?;
        Ok(games.into_iter().find(|g| g.appid == appid))
    }

    /// Find games by name (case-insensitive partial match)
    pub fn find_by_name(&self, name: &str) -> Result<Vec<SteamGame>> {
        let name_lower = name.to_lowercase();
        let games = self.installed_games()?;
        Ok(games
            .into_iter()
            .filter(|g| g.name.to_lowercase().contains(&name_lower))
            .collect())
    }

    /// Get launch options for all games
    /// Returns a map of AppID -> launch options string
    pub fn get_all_launch_options(&self) -> Result<HashMap<u32, String>> {
        let mut options = HashMap::new();
        
        // Find userdata folder
        let userdata_path = self.steam_path.join("userdata");
        if !userdata_path.exists() {
            return Ok(options);
        }

        // Iterate through user folders
        for user_entry in std::fs::read_dir(&userdata_path)?.filter_map(|e| e.ok()) {
            let user_path = user_entry.path();
            if !user_path.is_dir() {
                continue;
            }

            let localconfig = user_path.join("config").join("localconfig.vdf");
            if localconfig.exists() {
                if let Ok(user_options) = parse_launch_options(&localconfig) {
                    options.extend(user_options);
                }
            }
        }

        Ok(options)
    }

    /// Get launch options for a specific game
    pub fn get_launch_options(&self, appid: u32) -> Result<Option<String>> {
        let all_options = self.get_all_launch_options()?;
        Ok(all_options.get(&appid).cloned())
    }
}

/// Detect Steam installation path
fn detect_steam_path() -> Result<PathBuf> {
    // Try common locations on Windows
    let candidates = [
        // Default install location
        PathBuf::from(r"C:\Program Files (x86)\Steam"),
        PathBuf::from(r"C:\Program Files\Steam"),
        // User's home directory
        dirs::home_dir()
            .map(|h| h.join("Steam"))
            .unwrap_or_default(),
    ];

    for path in candidates {
        if path.join("steam.exe").exists() || path.join("steamapps").exists() {
            return Ok(path);
        }
    }

    // Try reading from registry
    #[cfg(windows)]
    {
        if let Ok(path) = read_steam_path_from_registry() {
            return Ok(path);
        }
    }

    anyhow::bail!("Could not find Steam installation")
}

#[cfg(windows)]
fn read_steam_path_from_registry() -> Result<PathBuf> {
    use std::process::Command;

    // Use reg query to read Steam path
    let output = Command::new("reg")
        .args([
            "query",
            r"HKCU\Software\Valve\Steam",
            "/v",
            "SteamPath",
        ])
        .output()
        .context("Failed to query registry")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    for line in stdout.lines() {
        if line.contains("SteamPath") {
            if let Some(path_start) = line.find("REG_SZ") {
                let path = line[path_start + 6..].trim();
                return Ok(PathBuf::from(path));
            }
        }
    }

    anyhow::bail!("Steam path not found in registry")
}

/// Parse an appmanifest_*.acf file
fn parse_app_manifest(path: &PathBuf, steamapps_folder: &PathBuf) -> Result<SteamGame> {
    let content = std::fs::read_to_string(path)
        .context("Failed to read app manifest")?;

    let mut values: HashMap<&str, String> = HashMap::new();

    for line in content.lines() {
        let line = line.trim();
        // Parse simple key-value pairs
        if let Some((key, value)) = parse_vdf_line(line) {
            values.insert(key, value);
        }
    }

    let appid = values.get("appid")
        .and_then(|s| s.parse::<u32>().ok())
        .context("Missing appid in manifest")?;

    let name = values.get("name")
        .cloned()
        .context("Missing name in manifest")?;

    let install_dir = values.get("installdir")
        .map(|dir| steamapps_folder.join("common").join(dir))
        .context("Missing installdir in manifest")?;

    let size_bytes = values.get("SizeOnDisk")
        .and_then(|s| s.parse::<u64>().ok());

    Ok(SteamGame {
        appid,
        name,
        install_dir,
        size_bytes,
        launch_options: None, // Populated separately via get_all_launch_options
    })
}

/// Parse a VDF key-value line like: "key" "value"
fn parse_vdf_line(line: &str) -> Option<(&str, String)> {
    let line = line.trim();
    if !line.starts_with('"') {
        return None;
    }

    let parts: Vec<&str> = line.split('"').filter(|s| !s.trim().is_empty()).collect();
    if parts.len() >= 2 {
        Some((parts[0], parts[1].to_string()))
    } else {
        None
    }
}

/// Extract value from VDF line like: "path" "C:\Games\Steam"
fn extract_vdf_value(line: &str) -> Option<String> {
    let parts: Vec<&str> = line.split('"').collect();
    if parts.len() >= 4 {
        Some(parts[3].to_string())
    } else {
        None
    }
}

/// Parse launch options from localconfig.vdf
fn parse_launch_options(path: &PathBuf) -> Result<HashMap<u32, String>> {
    let content = std::fs::read_to_string(path)
        .context("Failed to read localconfig.vdf")?;
    
    let mut options = HashMap::new();
    let mut current_appid: Option<u32> = None;
    let mut in_apps_section = false;
    let mut brace_depth = 0;
    let mut in_app_block = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Track brace depth
        if trimmed == "{" {
            brace_depth += 1;
        } else if trimmed == "}" {
            brace_depth -= 1;
            if in_app_block && brace_depth <= 2 {
                in_app_block = false;
                current_appid = None;
            }
            if in_apps_section && brace_depth <= 1 {
                in_apps_section = false;
            }
        }

        // Look for "apps" or "Apps" section
        if trimmed == "\"apps\"" || trimmed == "\"Apps\"" {
            in_apps_section = true;
            continue;
        }

        if in_apps_section {
            // Check for app ID (numeric key)
            if let Some(appid_str) = trimmed.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
                if let Ok(appid) = appid_str.parse::<u32>() {
                    current_appid = Some(appid);
                    in_app_block = true;
                }
            }

            // Check for LaunchOptions within an app block
            if in_app_block && current_appid.is_some() {
                if trimmed.starts_with("\"LaunchOptions\"") {
                    if let Some(value) = extract_vdf_value(trimmed) {
                        if !value.is_empty() {
                            options.insert(current_appid.unwrap(), value);
                        }
                    }
                }
            }
        }
    }

    Ok(options)
}

impl SteamGame {
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
