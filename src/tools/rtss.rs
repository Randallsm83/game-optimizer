//! RTSS (RivaTuner Statistics Server) integration
//!
//! Manages RTSS application profiles for framerate limiting.
//! Uses rtss-sys crate for SDK bindings when available,
//! falls back to profile file manipulation.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// RTSS manager
pub struct RtssManager {
    /// Path to RTSS installation
    install_path: Option<PathBuf>,
    /// Path to profiles directory
    profiles_path: Option<PathBuf>,
}

impl RtssManager {
    /// Create a new RTSS manager with auto-detected paths
    pub fn new() -> Result<Self> {
        let install_path = detect_rtss_path();
        let profiles_path = install_path.as_ref().map(|p| p.join("Profiles"));
        
        Ok(Self {
            install_path,
            profiles_path,
        })
    }

    /// Check if RTSS is installed
    pub fn is_installed(&self) -> bool {
        self.install_path.is_some()
    }

    /// Get RTSS installation path
    pub fn install_path(&self) -> Option<&PathBuf> {
        self.install_path.as_ref()
    }

    /// Check if RTSS is currently running
    pub fn is_running(&self) -> bool {
        #[cfg(windows)]
        {
            use std::process::Command;
            
            if let Ok(output) = Command::new("tasklist")
                .args(["/FI", "IMAGENAME eq RTSS.exe"])
                .output()
            {
                let stdout = String::from_utf8_lossy(&output.stdout);
                return stdout.contains("RTSS.exe");
            }
        }
        false
    }

    /// List all RTSS profiles
    pub fn list_profiles(&self) -> Result<Vec<RtssProfile>> {
        let profiles_path = self.profiles_path.as_ref()
            .ok_or_else(|| anyhow::anyhow!("RTSS not installed"))?;
        
        if !profiles_path.exists() {
            return Ok(Vec::new());
        }

        let mut profiles = Vec::new();
        
        for entry in std::fs::read_dir(profiles_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().map(|e| e == "cfg").unwrap_or(false) {
                if let Ok(profile) = parse_rtss_profile(&path) {
                    profiles.push(profile);
                }
            }
        }

        profiles.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        Ok(profiles)
    }

    /// Get profile for a specific game/application
    pub fn get_profile(&self, name: &str) -> Result<Option<RtssProfile>> {
        let profiles = self.list_profiles()?;
        Ok(profiles.into_iter().find(|p| {
            p.name.to_lowercase().contains(&name.to_lowercase()) ||
            p.executable.to_lowercase().contains(&name.to_lowercase())
        }))
    }

    /// Set framerate limit for an application
    pub fn set_fps_limit(&self, executable: &str, fps_limit: u32) -> Result<()> {
        let profiles_path = self.profiles_path.as_ref()
            .ok_or_else(|| anyhow::anyhow!("RTSS not installed"))?;

        // Create profile filename from executable
        let profile_name = executable
            .trim_end_matches(".exe")
            .replace(['/', '\\', ':'], "_");
        let profile_path = profiles_path.join(format!("{}.cfg", profile_name));

        // Load existing profile or create new one
        let mut profile = if profile_path.exists() {
            parse_rtss_profile(&profile_path).unwrap_or_else(|_| RtssProfile {
                name: profile_name.clone(),
                executable: executable.to_string(),
                ..Default::default()
            })
        } else {
            RtssProfile {
                name: profile_name.clone(),
                executable: executable.to_string(),
                ..Default::default()
            }
        };

        profile.framerate_limit = Some(fps_limit);
        
        // Save profile
        save_rtss_profile(&profile_path, &profile)?;
        
        Ok(())
    }

    /// Remove FPS limit for an application
    pub fn remove_fps_limit(&self, executable: &str) -> Result<()> {
        self.set_fps_limit(executable, 0)
    }

    /// Get global RTSS settings
    pub fn get_global_settings(&self) -> Result<RtssGlobalSettings> {
        let install_path = self.install_path.as_ref()
            .ok_or_else(|| anyhow::anyhow!("RTSS not installed"))?;
        
        let settings_path = install_path.join("RTSS.cfg");
        
        if settings_path.exists() {
            parse_rtss_global_settings(&settings_path)
        } else {
            Ok(RtssGlobalSettings::default())
        }
    }
}

/// An RTSS application profile
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RtssProfile {
    /// Profile name
    pub name: String,
    /// Executable name (e.g., "game.exe")
    pub executable: String,
    /// Framerate limit (0 = disabled)
    pub framerate_limit: Option<u32>,
    /// Scanline sync (0 = disabled)
    pub scanline_sync: Option<i32>,
    /// On-screen display enabled
    pub osd_enabled: bool,
    /// Show FPS in OSD
    pub show_fps: bool,
    /// Show frametime in OSD
    pub show_frametime: bool,
}

impl RtssProfile {
    /// Format framerate limit for display
    pub fn fps_display(&self) -> String {
        match self.framerate_limit {
            Some(0) | None => "Unlimited".to_string(),
            Some(fps) => format!("{} FPS", fps),
        }
    }
}

/// Global RTSS settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RtssGlobalSettings {
    /// Default framerate limit
    pub default_fps_limit: Option<u32>,
    /// Start with Windows
    pub start_with_windows: bool,
    /// Start minimized
    pub start_minimized: bool,
}

/// Detect RTSS installation path
fn detect_rtss_path() -> Option<PathBuf> {
    // Common installation paths
    let candidates = [
        PathBuf::from(r"C:\Program Files (x86)\RivaTuner Statistics Server"),
        PathBuf::from(r"C:\Program Files\RivaTuner Statistics Server"),
        // MSI Afterburner bundle location
        PathBuf::from(r"C:\Program Files (x86)\MSI Afterburner\RTSS"),
    ];

    for path in candidates {
        if path.join("RTSS.exe").exists() {
            return Some(path);
        }
    }

    // Try registry
    #[cfg(windows)]
    {
        use std::process::Command;
        
        // Try common registry locations
        let reg_paths = [
            r"HKLM\SOFTWARE\WOW6432Node\Unwinder\RTSS",
            r"HKLM\SOFTWARE\Unwinder\RTSS",
        ];

        for reg_path in reg_paths {
            if let Ok(output) = Command::new("reg")
                .args(["query", reg_path, "/v", "InstallPath"])
                .output()
            {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if line.contains("InstallPath") {
                        if let Some(idx) = line.find("REG_SZ") {
                            let path = PathBuf::from(line[idx + 6..].trim());
                            if path.exists() {
                                return Some(path);
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

/// Parse an RTSS profile from a .cfg file
fn parse_rtss_profile(path: &PathBuf) -> Result<RtssProfile> {
    let content = std::fs::read_to_string(path)
        .context("Failed to read RTSS profile")?;
    
    let name = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();
    
    let mut profile = RtssProfile {
        name,
        ..Default::default()
    };

    for line in content.lines() {
        let line = line.trim();
        if let Some((key, value)) = line.split_once('=') {
            match key.trim() {
                "AppName" | "Application" => {
                    profile.executable = value.trim().trim_matches('"').to_string();
                }
                "FramerateLimit" | "FPSLimit" => {
                    profile.framerate_limit = value.trim().parse().ok();
                }
                "ScanlineSync" => {
                    profile.scanline_sync = value.trim().parse().ok();
                }
                "OSDX" | "EnableOSD" => {
                    profile.osd_enabled = value.trim() == "1";
                }
                "ShowFPS" => {
                    profile.show_fps = value.trim() == "1";
                }
                "ShowFrametime" => {
                    profile.show_frametime = value.trim() == "1";
                }
                _ => {}
            }
        }
    }

    Ok(profile)
}

/// Save an RTSS profile to a .cfg file
fn save_rtss_profile(path: &PathBuf, profile: &RtssProfile) -> Result<()> {
    let mut content = String::new();
    
    content.push_str(&format!("[{}]\n", profile.name));
    content.push_str(&format!("AppName={}\n", profile.executable));
    
    if let Some(fps) = profile.framerate_limit {
        content.push_str(&format!("FramerateLimit={}\n", fps));
    }
    
    if let Some(scanline) = profile.scanline_sync {
        content.push_str(&format!("ScanlineSync={}\n", scanline));
    }
    
    content.push_str(&format!("EnableOSD={}\n", if profile.osd_enabled { 1 } else { 0 }));
    content.push_str(&format!("ShowFPS={}\n", if profile.show_fps { 1 } else { 0 }));
    content.push_str(&format!("ShowFrametime={}\n", if profile.show_frametime { 1 } else { 0 }));

    std::fs::write(path, content)?;
    Ok(())
}

/// Parse global RTSS settings
fn parse_rtss_global_settings(path: &PathBuf) -> Result<RtssGlobalSettings> {
    let content = std::fs::read_to_string(path)
        .context("Failed to read RTSS settings")?;
    
    let mut settings = RtssGlobalSettings::default();

    for line in content.lines() {
        let line = line.trim();
        if let Some((key, value)) = line.split_once('=') {
            match key.trim() {
                "FramerateLimit" => {
                    settings.default_fps_limit = value.trim().parse().ok();
                }
                "StartWithWindows" => {
                    settings.start_with_windows = value.trim() == "1";
                }
                "StartMinimized" => {
                    settings.start_minimized = value.trim() == "1";
                }
                _ => {}
            }
        }
    }

    Ok(settings)
}

/// Calculate optimal RTSS FPS limit for G-Sync/FreeSync
pub fn calculate_vrr_fps_limit(refresh_rate: u32) -> u32 {
    // Cap 3 below refresh rate to stay in VRR range
    refresh_rate.saturating_sub(3)
}

/// Recommendation for RTSS settings
#[derive(Debug, Clone)]
pub struct RtssRecommendation {
    pub fps_limit: Option<u32>,
    pub scanline_sync: Option<i32>,
    pub reason: String,
}

/// Generate RTSS recommendation based on display and preferences
pub fn recommend_rtss(
    target_fps: u32,
    refresh_rate: u32,
    vrr_enabled: bool,
    use_scanline_sync: bool,
) -> RtssRecommendation {
    if vrr_enabled {
        let fps_limit = calculate_vrr_fps_limit(refresh_rate);
        RtssRecommendation {
            fps_limit: Some(fps_limit),
            scanline_sync: None,
            reason: format!(
                "Cap at {} FPS to stay within G-Sync/FreeSync range ({}Hz)",
                fps_limit, refresh_rate
            ),
        }
    } else if use_scanline_sync {
        // Scanline sync for tear-free without VRR
        RtssRecommendation {
            fps_limit: Some(target_fps),
            scanline_sync: Some(-10), // Negative = before VBlank
            reason: format!(
                "Scanline sync at {} FPS for tear-free output",
                target_fps
            ),
        }
    } else {
        // Simple FPS cap
        RtssRecommendation {
            fps_limit: Some(target_fps),
            scanline_sync: None,
            reason: format!("Cap at {} FPS for consistent frame pacing", target_fps),
        }
    }
}
