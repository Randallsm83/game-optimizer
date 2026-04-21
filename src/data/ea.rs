//! EA App game detection
//!
//! Reads installed games from EA App's local content data

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// An EA App game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EaGame {
    pub name: String,
    pub content_id: String,
    pub install_path: PathBuf,
    pub locale: Option<String>,
}

/// EA App library manager
pub struct EaLibrary {
    data_path: PathBuf,
}

impl EaLibrary {
    /// Create a new EA library manager with auto-detected path
    pub fn new() -> Result<Self> {
        let data_path = detect_ea_data_path()?;
        Ok(Self { data_path })
    }

    /// Get all installed EA games
    pub fn installed_games(&self) -> Result<Vec<EaGame>> {
        let mut games = Vec::new();

        // EA App stores install info in XML files at:
        // %ProgramData%\EA Desktop\InstallData\<content_id>\
        let install_data = self.data_path.join("InstallData");
        
        if !install_data.exists() {
            // Also check legacy Origin path
            return self.scan_origin_games();
        }

        if let Ok(entries) = std::fs::read_dir(&install_data) {
            for entry in entries.filter_map(|e| e.ok()) {
                let content_path = entry.path();
                if content_path.is_dir() {
                    // Look for installer data XML
                    let installer_data = content_path.join("installerdata.xml");
                    if installer_data.exists() {
                        if let Ok(game) = parse_ea_installer_data(&installer_data) {
                            games.push(game);
                        }
                    }
                }
            }
        }

        games.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        Ok(games)
    }

    /// Scan legacy Origin install locations
    fn scan_origin_games(&self) -> Result<Vec<EaGame>> {
        let mut games = Vec::new();

        // Check Origin's local content
        let origin_path = dirs::data_local_dir()
            .unwrap_or_default()
            .join("Origin")
            .join("LocalContent");

        if origin_path.exists() {
            if let Ok(entries) = std::fs::read_dir(&origin_path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let game_path = entry.path();
                    if game_path.is_dir() {
                        // Look for .mfst files
                        if let Ok(files) = std::fs::read_dir(&game_path) {
                            for file in files.filter_map(|f| f.ok()) {
                                let file_path = file.path();
                                if file_path.extension().map(|e| e == "mfst").unwrap_or(false) {
                                    if let Ok(game) = parse_origin_manifest(&file_path) {
                                        games.push(game);
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

    /// Find a game by name
    pub fn find_by_name(&self, name: &str) -> Result<Vec<EaGame>> {
        let name_lower = name.to_lowercase();
        let games = self.installed_games()?;
        Ok(games
            .into_iter()
            .filter(|g| g.name.to_lowercase().contains(&name_lower))
            .collect())
    }
}

/// Parse EA Desktop installer data XML
fn parse_ea_installer_data(path: &PathBuf) -> Result<EaGame> {
    let content = std::fs::read_to_string(path)?;
    
    // Simple XML parsing for key fields (avoiding full XML parser dependency)
    let content_id = extract_xml_value(&content, "contentId")
        .unwrap_or_else(|| "unknown".to_string());
    let name = extract_xml_value(&content, "displayName")
        .or_else(|| extract_xml_value(&content, "gameTitle"))
        .unwrap_or_else(|| content_id.clone());
    let install_path = extract_xml_value(&content, "filePath")
        .or_else(|| extract_xml_value(&content, "installDir"))
        .map(PathBuf::from)
        .unwrap_or_default();
    let locale = extract_xml_value(&content, "locale");

    Ok(EaGame {
        name,
        content_id,
        install_path,
        locale,
    })
}

/// Parse Origin manifest file
fn parse_origin_manifest(path: &PathBuf) -> Result<EaGame> {
    let content = std::fs::read_to_string(path)?;
    
    // Origin .mfst files are URL-encoded key=value pairs
    let mut name = String::new();
    let mut content_id = String::new();
    let mut install_path = PathBuf::new();

    for part in content.split('&') {
        if let Some((key, value)) = part.split_once('=') {
            let decoded = urlencoding::decode(value).unwrap_or_default();
            match key {
                "dipinstallpath" => install_path = PathBuf::from(decoded.as_ref()),
                "id" => content_id = decoded.to_string(),
                _ => {}
            }
        }
    }

    // Derive name from install path if not found
    if name.is_empty() {
        name = install_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&content_id)
            .to_string();
    }

    Ok(EaGame {
        name,
        content_id,
        install_path,
        locale: None,
    })
}

/// Simple XML value extractor
fn extract_xml_value(content: &str, tag: &str) -> Option<String> {
    let open_tag = format!("<{}>", tag);
    let close_tag = format!("</{}>", tag);
    
    if let Some(start) = content.find(&open_tag) {
        let value_start = start + open_tag.len();
        if let Some(end) = content[value_start..].find(&close_tag) {
            return Some(content[value_start..value_start + end].trim().to_string());
        }
    }
    
    // Try attribute style: tag="value"
    let attr_pattern = format!("{}=\"", tag);
    if let Some(start) = content.find(&attr_pattern) {
        let value_start = start + attr_pattern.len();
        if let Some(end) = content[value_start..].find('"') {
            return Some(content[value_start..value_start + end].to_string());
        }
    }
    
    None
}

/// Detect EA App data path
fn detect_ea_data_path() -> Result<PathBuf> {
    let program_data = std::env::var("ProgramData")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(r"C:\ProgramData"));

    Ok(program_data.join("EA Desktop"))
}
