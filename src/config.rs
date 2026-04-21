//! Application configuration management

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Claude API key for AI recommendations
    pub claude_api_key: Option<String>,

    /// Steam installation path (auto-detected if None)
    pub steam_path: Option<PathBuf>,

    /// RTSS installation path (auto-detected if None)
    pub rtss_path: Option<PathBuf>,

    /// Lossless Scaling config path (auto-detected if None)
    pub lossless_scaling_path: Option<PathBuf>,

    /// Default target resolution
    pub target_resolution: Option<String>,

    /// Default target refresh rate
    pub target_refresh_rate: Option<u32>,

    /// Enable G-Sync by default
    pub gsync_enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            claude_api_key: None,
            steam_path: None,
            rtss_path: None,
            lossless_scaling_path: None,
            target_resolution: Some("7680x2160".to_string()),
            target_refresh_rate: Some(244),
            gsync_enabled: true,
        }
    }
}

impl Config {
    /// Get the config file path (XDG compliant: ~/.config/game-optimizer/config.toml)
    pub fn config_path() -> PathBuf {
        Self::xdg_config_dir()
            .join("game-optimizer")
            .join("config.toml")
    }

    /// Get the data directory path (XDG compliant: ~/.local/share/game-optimizer)
    pub fn data_dir() -> PathBuf {
        Self::xdg_data_dir()
            .join("game-optimizer")
    }

    /// Get the cache directory path (XDG compliant: ~/.cache/game-optimizer)
    pub fn cache_dir() -> PathBuf {
        Self::xdg_cache_dir()
            .join("game-optimizer")
    }

    /// Get XDG_CONFIG_HOME or ~/.config
    fn xdg_config_dir() -> PathBuf {
        std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join(".config")
            })
    }

    /// Get XDG_DATA_HOME or ~/.local/share
    fn xdg_data_dir() -> PathBuf {
        std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join(".local")
                    .join("share")
            })
    }

    /// Get XDG_CACHE_HOME or ~/.cache
    fn xdg_cache_dir() -> PathBuf {
        std::env::var("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join(".cache")
            })
    }

    /// Load configuration from file
    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        if path.exists() {
            let contents = std::fs::read_to_string(&path)?;
            let config: Config = toml::from_str(&contents)?;
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let contents = toml::to_string_pretty(self)?;
        std::fs::write(&path, contents)?;
        Ok(())
    }
}
