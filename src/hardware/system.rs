//! System hardware detection (CPU, RAM, monitors)

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// System information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// CPU name
    pub cpu_name: Option<String>,

    /// Total RAM in bytes
    pub ram_bytes: u64,

    /// RAM in human-readable format
    pub ram_display: String,

    /// Primary monitor resolution
    pub monitor_resolution: Option<String>,

    /// Primary monitor refresh rate
    pub monitor_refresh_rate: Option<u32>,

    /// Windows version
    pub os_version: String,
}

impl SystemInfo {
    /// Detect system hardware information
    pub fn detect() -> Result<Self> {
        // For now, return placeholder values
        // Full implementation would use WMI or Windows APIs
        Ok(SystemInfo {
            cpu_name: None, // Would use WMI to detect
            ram_bytes: 0,
            ram_display: "Unknown".to_string(),
            monitor_resolution: None,
            monitor_refresh_rate: None,
            os_version: std::env::var("OS").unwrap_or_else(|_| "Windows".to_string()),
        })
    }
}

/// Monitor information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorInfo {
    /// Monitor name/model
    pub name: String,

    /// Resolution width
    pub width: u32,

    /// Resolution height
    pub height: u32,

    /// Refresh rate in Hz
    pub refresh_rate: u32,

    /// Whether G-Sync is supported
    pub gsync_supported: bool,

    /// Whether HDR is supported
    pub hdr_supported: bool,
}

/// Enumerate connected monitors
pub fn enumerate_monitors() -> Result<Vec<MonitorInfo>> {
    // Placeholder - would use Windows display APIs
    Ok(Vec::new())
}
