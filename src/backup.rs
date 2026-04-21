//! Backup and restore functionality for game optimization settings
//!
//! Stores backups in ~/.local/share/game-optimizer/backups/

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::ai::prompts::{DriverSettings, LosslessScalingSettings};

/// A backup of game optimization settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsBackup {
    /// Backup ID (timestamp-based)
    pub id: String,
    /// When the backup was created
    pub created_at: DateTime<Utc>,
    /// Game name this backup is for
    pub game_name: String,
    /// Game executable
    pub executable: Option<String>,
    /// NVIDIA driver settings before modification
    pub nvidia_settings: Option<NvidiaBackup>,
    /// RTSS settings before modification
    pub rtss_settings: Option<RtssBackup>,
    /// Lossless Scaling settings before modification
    pub lossless_scaling_settings: Option<LsBackup>,
}

/// Backup of NVIDIA profile settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NvidiaBackup {
    pub profile_name: String,
    pub existed: bool,
    pub settings: DriverSettings,
}

/// Backup of RTSS profile settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RtssBackup {
    pub profile_name: String,
    pub existed: bool,
    pub fps_limit: Option<u32>,
}

/// Backup of Lossless Scaling profile settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LsBackup {
    pub profile_name: String,
    pub existed: bool,
    pub settings: Option<LosslessScalingSettings>,
}

/// Manages backup storage and restoration
pub struct BackupManager {
    /// Directory where backups are stored
    backup_dir: PathBuf,
}

impl BackupManager {
    /// Create a new backup manager
    pub fn new() -> Result<Self> {
        let backup_dir = get_backup_dir()?;
        std::fs::create_dir_all(&backup_dir)
            .context("Failed to create backup directory")?;
        
        Ok(Self { backup_dir })
    }

    /// Get the backup directory path
    pub fn backup_dir(&self) -> &PathBuf {
        &self.backup_dir
    }

    /// Create a new backup
    pub fn create_backup(&self, backup: &SettingsBackup) -> Result<PathBuf> {
        let filename = format!("{}.json", backup.id);
        let path = self.backup_dir.join(&filename);
        
        let json = serde_json::to_string_pretty(backup)
            .context("Failed to serialize backup")?;
        std::fs::write(&path, json)
            .context("Failed to write backup file")?;
        
        tracing::info!("Created backup: {}", filename);
        Ok(path)
    }

    /// List all backups
    pub fn list_backups(&self) -> Result<Vec<SettingsBackup>> {
        let mut backups = Vec::new();
        
        if !self.backup_dir.exists() {
            return Ok(backups);
        }
        
        for entry in std::fs::read_dir(&self.backup_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(backup) = serde_json::from_str::<SettingsBackup>(&content) {
                        backups.push(backup);
                    }
                }
            }
        }
        
        // Sort by creation time, newest first
        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(backups)
    }

    /// List backups for a specific game
    pub fn list_backups_for_game(&self, game_name: &str) -> Result<Vec<SettingsBackup>> {
        let search = game_name.to_lowercase();
        let backups = self.list_backups()?;
        
        Ok(backups
            .into_iter()
            .filter(|b| b.game_name.to_lowercase().contains(&search))
            .collect())
    }

    /// Get a specific backup by ID
    pub fn get_backup(&self, id: &str) -> Result<Option<SettingsBackup>> {
        let path = self.backup_dir.join(format!("{}.json", id));
        
        if !path.exists() {
            return Ok(None);
        }
        
        let content = std::fs::read_to_string(&path)
            .context("Failed to read backup file")?;
        let backup = serde_json::from_str(&content)
            .context("Failed to parse backup file")?;
        
        Ok(Some(backup))
    }

    /// Get the most recent backup for a game
    pub fn get_latest_backup(&self, game_name: &str) -> Result<Option<SettingsBackup>> {
        let backups = self.list_backups_for_game(game_name)?;
        Ok(backups.into_iter().next())
    }

    /// Delete a backup by ID
    pub fn delete_backup(&self, id: &str) -> Result<bool> {
        let path = self.backup_dir.join(format!("{}.json", id));
        
        if path.exists() {
            std::fs::remove_file(&path)?;
            tracing::info!("Deleted backup: {}", id);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Generate a new backup ID
    pub fn generate_id(game_name: &str) -> String {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let clean_name: String = game_name
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
            .take(30)
            .collect();
        format!("{}_{}", clean_name, timestamp)
    }
}

/// Get the backup directory path
fn get_backup_dir() -> Result<PathBuf> {
    // Use XDG data directory on all platforms
    let data_dir = if cfg!(windows) {
        // Windows: use %LOCALAPPDATA%
        std::env::var("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."))
            })
    } else {
        // Unix: use XDG_DATA_HOME or ~/.local/share
        dirs::data_dir().unwrap_or_else(|| {
            dirs::home_dir()
                .map(|h| h.join(".local").join("share"))
                .unwrap_or_else(|| PathBuf::from("."))
        })
    };
    
    Ok(data_dir.join("game-optimizer").join("backups"))
}

/// Result of applying optimizations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyResult {
    /// Whether NVIDIA settings were applied
    pub nvidia_applied: bool,
    /// Whether RTSS settings were applied
    pub rtss_applied: bool,
    /// Whether Lossless Scaling settings were applied
    pub ls_applied: bool,
    /// Backup ID if backup was created
    pub backup_id: Option<String>,
    /// Any warnings during application
    pub warnings: Vec<String>,
}

impl Default for ApplyResult {
    fn default() -> Self {
        Self {
            nvidia_applied: false,
            rtss_applied: false,
            ls_applied: false,
            backup_id: None,
            warnings: Vec::new(),
        }
    }
}

/// Exportable game optimization profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameProfile {
    /// Profile format version
    pub version: String,
    /// Game name
    pub game_name: String,
    /// Game executable (if known)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executable: Option<String>,
    /// Target hardware this was optimized for
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_hardware: Option<String>,
    /// Target resolution
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_resolution: Option<String>,
    /// Target FPS
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_fps: Option<u32>,
    /// NVIDIA driver settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nvidia_settings: Option<DriverSettings>,
    /// RTSS FPS limit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rtss_fps_limit: Option<u32>,
    /// Lossless Scaling settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lossless_scaling: Option<LosslessScalingSettings>,
    /// Notes or comments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    /// When the profile was created/exported
    pub created_at: DateTime<Utc>,
}

impl GameProfile {
    pub fn new(game_name: &str) -> Self {
        Self {
            version: "1.0".to_string(),
            game_name: game_name.to_string(),
            executable: None,
            target_hardware: None,
            target_resolution: None,
            target_fps: None,
            nvidia_settings: None,
            rtss_fps_limit: None,
            lossless_scaling: None,
            notes: None,
            created_at: Utc::now(),
        }
    }

    /// Export to JSON string
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).context("Failed to serialize profile to JSON")
    }

    /// Export to TOML string
    pub fn to_toml(&self) -> Result<String> {
        toml::to_string_pretty(self).context("Failed to serialize profile to TOML")
    }

    /// Import from JSON string
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).context("Failed to parse JSON profile")
    }

    /// Import from TOML string
    pub fn from_toml(toml_str: &str) -> Result<Self> {
        toml::from_str(toml_str).context("Failed to parse TOML profile")
    }

    /// Import from file (auto-detect format)
    pub fn from_file(path: &std::path::Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;
        
        // Try to detect format from extension or content
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        match ext.to_lowercase().as_str() {
            "toml" => Self::from_toml(&content),
            "json" | _ => Self::from_json(&content),
        }
    }

    /// Export to file
    pub fn to_file(&self, path: &std::path::Path, format: &str) -> Result<()> {
        let content = match format.to_lowercase().as_str() {
            "toml" => self.to_toml()?,
            _ => self.to_json()?,
        };
        std::fs::write(path, content)
            .with_context(|| format!("Failed to write file: {}", path.display()))?;
        Ok(())
    }
}
