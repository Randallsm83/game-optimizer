//! Lossless Scaling profile management
//!
//! Manages Lossless Scaling configuration and per-game profiles.
//! Config location: %LOCALAPPDATA%\Lossless Scaling\Settings.xml

use anyhow::{Context, Result};
use quick_xml::de::from_str;
use quick_xml::se::to_string;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Lossless Scaling manager
pub struct LosslessScaling {
    /// Path to Settings.xml
    settings_path: PathBuf,
    /// Path to LS installation (for config.ini)
    install_path: Option<PathBuf>,
}

impl LosslessScaling {
    /// Create a new Lossless Scaling manager with auto-detected paths
    pub fn new() -> Result<Self> {
        let settings_path = get_settings_path()?;
        let install_path = detect_ls_install_path();

        Ok(Self {
            settings_path,
            install_path,
        })
    }

    /// Get the settings file path
    pub fn settings_path(&self) -> &PathBuf {
        &self.settings_path
    }

    /// Check if Lossless Scaling settings exist
    pub fn has_settings(&self) -> bool {
        self.settings_path.exists()
    }

    /// Load all settings from Settings.xml
    pub fn load_settings(&self) -> Result<LsSettings> {
        if !self.settings_path.exists() {
            return Ok(LsSettings::default());
        }

        let content = std::fs::read_to_string(&self.settings_path)
            .context("Failed to read Lossless Scaling settings")?;

        from_str(&content).context("Failed to parse Lossless Scaling settings")
    }

    /// Save settings to Settings.xml
    pub fn save_settings(&self, settings: &LsSettings) -> Result<()> {
        // Ensure directory exists
        if let Some(parent) = self.settings_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let xml = to_string(settings).context("Failed to serialize settings")?;
        let xml = format!("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n{}", xml);
        std::fs::write(&self.settings_path, xml)?;

        Ok(())
    }

    /// Get all saved profiles
    pub fn list_profiles(&self) -> Result<Vec<LsProfile>> {
        let settings = self.load_settings()?;
        Ok(settings.game_profiles.profiles)
    }

    /// Get a profile for a specific game
    pub fn get_profile(&self, game_name: &str) -> Result<Option<LsProfile>> {
        let profiles = self.list_profiles()?;
        let search = game_name.to_lowercase();
        Ok(profiles.into_iter().find(|p| {
            p.title.to_lowercase().contains(&search)
                || p.path
                    .as_ref()
                    .map(|path| path.to_lowercase().contains(&search))
                    .unwrap_or(false)
        }))
    }

    /// Save or update a profile
    pub fn save_profile(&self, profile: LsProfile) -> Result<()> {
        let mut settings = self.load_settings()?;

        // Find existing profile by title
        if let Some(existing) = settings
            .game_profiles
            .profiles
            .iter_mut()
            .find(|p| p.title == profile.title)
        {
            *existing = profile;
        } else {
            settings.game_profiles.profiles.push(profile);
        }

        self.save_settings(&settings)
    }

    /// Get installation path if detected
    pub fn install_path(&self) -> Option<&PathBuf> {
        self.install_path.as_ref()
    }

    /// Calculate effective output FPS with LSFG
    /// base_fps * multiplier = output_fps
    pub fn calculate_output_fps(base_fps: u32, multiplier: LsfgMultiplier) -> u32 {
        base_fps * multiplier.value()
    }

    /// Calculate recommended base FPS for a target output
    /// target_fps / multiplier = base_fps
    pub fn calculate_base_fps(target_fps: u32, multiplier: LsfgMultiplier) -> u32 {
        target_fps / multiplier.value()
    }
}

/// Get the path to Settings.xml
fn get_settings_path() -> Result<PathBuf> {
    let local_app_data = std::env::var("LOCALAPPDATA")
        .context("LOCALAPPDATA environment variable not set")?;
    
    Ok(PathBuf::from(local_app_data)
        .join("Lossless Scaling")
        .join("Settings.xml"))
}

/// Detect Lossless Scaling installation path
fn detect_ls_install_path() -> Option<PathBuf> {
    // Check common Steam library locations
    let steam_path = get_steam_path().ok()?;
    let vdf_path = steam_path.join("steamapps").join("libraryfolders.vdf");

    let mut library_paths = vec![steam_path.join("steamapps")];

    // Parse libraryfolders.vdf for additional library paths
    if let Ok(content) = std::fs::read_to_string(&vdf_path) {
        for line in content.lines() {
            if line.contains("\"path\"") {
                let parts: Vec<&str> = line.split('"').collect();
                if parts.len() >= 4 {
                    let path = PathBuf::from(parts[3].replace("\\\\", "\\"));
                    library_paths.push(path.join("steamapps"));
                }
            }
        }
    }

    // Check each library for Lossless Scaling
    for lib in library_paths {
        let ls_path = lib.join("common").join("Lossless Scaling");
        if ls_path.exists() {
            return Some(ls_path);
        }
    }

    None
}

/// Get Steam installation path from registry
fn get_steam_path() -> Result<PathBuf> {
    #[cfg(windows)]
    {
        use std::process::Command;

        let output = Command::new("reg")
            .args(["query", r"HKCU\Software\Valve\Steam", "/v", "SteamPath"])
            .output()
            .context("Failed to query Steam registry")?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        for line in stdout.lines() {
            if line.contains("SteamPath") {
                if let Some(path_start) = line.find("REG_SZ") {
                    let path = line[path_start + 6..].trim();
                    return Ok(PathBuf::from(path));
                }
            }
        }
    }

    // Fallback to common locations
    let candidates = [
        PathBuf::from(r"C:\Program Files (x86)\Steam"),
        PathBuf::from(r"C:\Program Files\Steam"),
        PathBuf::from(r"C:\Steam"),
        PathBuf::from(r"C:\SteamGames"),
    ];

    for path in candidates {
        if path.exists() {
            return Ok(path);
        }
    }

    anyhow::bail!("Steam installation not found")
}

// ============================================================================
// Settings XML structures
// ============================================================================

/// Root settings structure for Settings.xml
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename = "Settings")]
pub struct LsSettings {
    #[serde(rename = "WindowMaximized", default)]
    pub window_maximized: bool,
    #[serde(rename = "WindowHeight", default)]
    pub window_height: f32,
    #[serde(rename = "WindowWidth", default)]
    pub window_width: f32,
    #[serde(rename = "ProfileListWidth", default)]
    pub profile_list_width: f32,
    #[serde(rename = "Hotkey", default)]
    pub hotkey: String,
    #[serde(rename = "HotkeyModifierKeys", default)]
    pub hotkey_modifier_keys: String,
    #[serde(rename = "GpuPreference", default)]
    pub gpu_preference: u32,
    #[serde(rename = "GpuPreferenceChangeCount", default)]
    pub gpu_preference_change_count: u32,
    #[serde(rename = "SetupPhase", default)]
    pub setup_phase: u32,
    #[serde(rename = "StartAsAdmin", default)]
    pub start_as_admin: bool,
    #[serde(rename = "StartAtWindowsStartup", default)]
    pub start_at_windows_startup: bool,
    #[serde(rename = "MinimizeToTray", default)]
    pub minimize_to_tray: bool,
    #[serde(rename = "CloseToTray", default)]
    pub close_to_tray: bool,
    #[serde(rename = "Language", default)]
    pub language: String,
    #[serde(rename = "Theme", default)]
    pub theme: String,
    #[serde(rename = "FrameGenerationCollapsed", default)]
    pub frame_generation_collapsed: bool,
    #[serde(rename = "CursorOptionsCollapsed", default)]
    pub cursor_options_collapsed: bool,
    #[serde(rename = "RenderingOptionsCollapsed", default)]
    pub rendering_options_collapsed: bool,
    #[serde(rename = "CaptureOptionsCollapsed", default)]
    pub capture_options_collapsed: bool,
    #[serde(rename = "GpuDisplayOptionsCollapsed", default)]
    pub gpu_display_options_collapsed: bool,
    #[serde(rename = "CropInputOptionsCollapsed", default)]
    pub crop_input_options_collapsed: bool,
    #[serde(rename = "BehaviorCollapsed", default)]
    pub behavior_collapsed: bool,
    #[serde(rename = "LegacyOptionsCollapsed", default)]
    pub legacy_options_collapsed: bool,
    #[serde(rename = "GameProfiles", default)]
    pub game_profiles: GameProfiles,
}

/// Container for game profiles
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameProfiles {
    #[serde(rename = "Profile", default)]
    pub profiles: Vec<LsProfile>,
}

/// A Lossless Scaling profile for a game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LsProfile {
    /// Profile title/name
    #[serde(rename = "Title")]
    pub title: String,

    /// Path to executable (optional, empty for Default profile)
    #[serde(rename = "Path", skip_serializing_if = "Option::is_none", default)]
    pub path: Option<String>,

    /// Auto-scale on launch
    #[serde(rename = "AutoScale", default)]
    pub auto_scale: bool,

    /// Auto-scale delay in seconds
    #[serde(rename = "AutoScaleDelay", default)]
    pub auto_scale_delay: u32,

    /// Scaling mode (Auto, Custom)
    #[serde(rename = "ScalingMode", default)]
    pub scaling_mode: String,

    /// Scaling fit mode (AspectRatio, Fullscreen)
    #[serde(rename = "ScalingFitMode", default)]
    pub scaling_fit_mode: String,

    /// Scale factor
    #[serde(rename = "ScaleFactor", default)]
    pub scale_factor: f32,

    /// Resize before scaling
    #[serde(rename = "ResizeBeforeScaling", default)]
    pub resize_before_scaling: bool,

    /// Windowed mode
    #[serde(rename = "WindowedMode", default)]
    pub windowed_mode: bool,

    /// Scaling type (Off, LS1, FSR, NIS, Integer)
    #[serde(rename = "ScalingType", default)]
    pub scaling_type: String,

    /// FSR type (ORIGINAL, OPTIMIZED)
    #[serde(rename = "FSRType", default)]
    pub fsr_type: String,

    /// LS1 type (BALANCED, PERFORMANCE, QUALITY)
    #[serde(rename = "LS1Type", default)]
    pub ls1_type: String,

    /// LSFG2 mode
    #[serde(rename = "LSFG2Mode", default)]
    pub lsfg2_mode: String,

    /// LSFG3 mode (FIXED, ADAPTIVE)
    #[serde(rename = "LSFG3Mode1", default)]
    pub lsfg3_mode: String,

    /// LSFG3 multiplier (2, 3, 4)
    #[serde(rename = "LSFG3Multiplier", default)]
    pub lsfg3_multiplier: u32,

    /// LSFG3 target FPS (for adaptive mode)
    #[serde(rename = "LSFG3Target", default)]
    pub lsfg3_target: u32,

    /// LSFG flow scale (0-100)
    #[serde(rename = "LSFGFlowScale", default)]
    pub lsfg_flow_scale: u32,

    /// LSFG size preset (BALANCED, PERFORMANCE, QUALITY)
    #[serde(rename = "LSFGSize", default)]
    pub lsfg_size: String,

    /// Anime4k type
    #[serde(rename = "Anime4kType", default)]
    pub anime4k_type: String,

    /// Sharpness (0-10)
    #[serde(rename = "Sharpness", default)]
    pub sharpness: u32,

    /// LS1 sharpness (0-10)
    #[serde(rename = "LS1Sharpness", default)]
    pub ls1_sharpness: u32,

    /// Variable Rate Shading
    #[serde(rename = "VRS", default)]
    pub vrs: bool,

    /// Frame generation mode (Off, LSFG2, LSFG3)
    #[serde(rename = "FrameGeneration", default)]
    pub frame_generation: String,

    /// Clip cursor to window
    #[serde(rename = "ClipCursor", default)]
    pub clip_cursor: bool,

    /// Adjust cursor speed
    #[serde(rename = "AdjustCursorSpeed", default)]
    pub adjust_cursor_speed: bool,

    /// Hide cursor
    #[serde(rename = "HideCursor", default)]
    pub hide_cursor: bool,

    /// Scale cursor
    #[serde(rename = "ScaleCursor", default)]
    pub scale_cursor: bool,

    /// Sync mode (OFF, VSYNC, etc)
    #[serde(rename = "SyncMode", default)]
    pub sync_mode: String,

    /// Max frame latency
    #[serde(rename = "MaxFrameLatency", default)]
    pub max_frame_latency: u32,

    /// G-Sync support
    #[serde(rename = "GsyncSupport", default)]
    pub gsync_support: bool,

    /// HDR support
    #[serde(rename = "HdrSupport", default)]
    pub hdr_support: bool,

    /// Draw FPS counter
    #[serde(rename = "DrawFps", default)]
    pub draw_fps: bool,

    /// Capture API (WGC, DXGI)
    #[serde(rename = "CaptureApi", default)]
    pub capture_api: String,

    /// Queue target for frame pacing
    #[serde(rename = "QueueTarget", default)]
    pub queue_target: u32,

    /// Preferred GPU ID
    #[serde(rename = "PreferredGpuId", default)]
    pub preferred_gpu_id: u32,

    /// Output display ID
    #[serde(rename = "OutputDisplayId", default)]
    pub output_display_id: u32,

    /// Crop input
    #[serde(rename = "CropInput", default)]
    pub crop_input: bool,

    /// Crop input left
    #[serde(rename = "CropInputLeft", default)]
    pub crop_input_left: u32,

    /// Crop input top
    #[serde(rename = "CropInputTop", default)]
    pub crop_input_top: u32,

    /// Crop input right
    #[serde(rename = "CropInputRight", default)]
    pub crop_input_right: u32,

    /// Crop input bottom
    #[serde(rename = "CropInputBottom", default)]
    pub crop_input_bottom: u32,

    /// Multi-display mode
    #[serde(rename = "MultiDisplayMode", default)]
    pub multi_display_mode: bool,
}

impl Default for LsProfile {
    fn default() -> Self {
        Self {
            title: String::new(),
            path: None,
            auto_scale: false,
            auto_scale_delay: 0,
            scaling_mode: "Auto".to_string(),
            scaling_fit_mode: "AspectRatio".to_string(),
            scale_factor: 2.0,
            resize_before_scaling: true,
            windowed_mode: false,
            scaling_type: "LS1".to_string(),
            fsr_type: "ORIGINAL".to_string(),
            ls1_type: "BALANCED".to_string(),
            lsfg2_mode: "X2".to_string(),
            lsfg3_mode: "FIXED".to_string(),
            lsfg3_multiplier: 2,
            lsfg3_target: 60,
            lsfg_flow_scale: 90,
            lsfg_size: "BALANCED".to_string(),
            anime4k_type: "S".to_string(),
            sharpness: 5,
            ls1_sharpness: 3,
            vrs: false,
            frame_generation: "LSFG3".to_string(),
            clip_cursor: true,
            adjust_cursor_speed: false,
            hide_cursor: false,
            scale_cursor: false,
            sync_mode: "OFF".to_string(),
            max_frame_latency: 1,
            gsync_support: false,
            hdr_support: false,
            draw_fps: false,
            capture_api: "WGC".to_string(),
            queue_target: 0,
            preferred_gpu_id: 0,
            output_display_id: 0,
            crop_input: false,
            crop_input_left: 0,
            crop_input_top: 0,
            crop_input_right: 0,
            crop_input_bottom: 0,
            multi_display_mode: false,
        }
    }
}

impl LsProfile {
    /// Get the frame generation mode as enum
    pub fn get_lsfg_mode(&self) -> LsfgMode {
        match self.frame_generation.as_str() {
            "LSFG3" => match self.lsfg3_mode.as_str() {
                "ADAPTIVE" => LsfgMode::Adaptive,
                _ => LsfgMode::Lsfg23,
            },
            "LSFG2" => LsfgMode::Lsfg10,
            _ => LsfgMode::Off,
        }
    }

    /// Get the multiplier as enum
    pub fn get_multiplier(&self) -> LsfgMultiplier {
        LsfgMultiplier::from_u32(self.lsfg3_multiplier)
    }

    /// Get scaling type as enum
    pub fn get_scaling_type(&self) -> ScalingType {
        match self.scaling_type.as_str() {
            "LS1" => ScalingType::Ls1,
            "FSR" => ScalingType::Fsr,
            "NIS" => ScalingType::Nis,
            "Integer" => ScalingType::Integer,
            _ => ScalingType::None,
        }
    }

    /// Format for display
    pub fn display_summary(&self) -> String {
        if self.frame_generation == "Off" {
            "Frame Gen: Off".to_string()
        } else {
            format!(
                "{} {}x @ {} FPS ({})",
                self.frame_generation,
                self.lsfg3_multiplier,
                self.lsfg3_target,
                self.lsfg3_mode
            )
        }
    }
}

// ============================================================================
// Enums for API compatibility
// ============================================================================

/// LSFG mode enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LsfgMode {
    #[default]
    Off,
    /// LSFG 2.3 (modern, recommended)
    Lsfg23,
    /// LSFG 1.0 (legacy)
    Lsfg10,
    /// Adaptive mode
    Adaptive,
}

impl std::fmt::Display for LsfgMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Off => write!(f, "Off"),
            Self::Lsfg23 => write!(f, "LSFG 2.3"),
            Self::Lsfg10 => write!(f, "LSFG 1.0"),
            Self::Adaptive => write!(f, "Adaptive"),
        }
    }
}

/// Frame generation multiplier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LsfgMultiplier {
    #[default]
    X2,
    X3,
    X4,
}

impl LsfgMultiplier {
    pub fn value(&self) -> u32 {
        match self {
            Self::X2 => 2,
            Self::X3 => 3,
            Self::X4 => 4,
        }
    }

    pub fn from_u32(v: u32) -> Self {
        match v {
            3 => Self::X3,
            4 => Self::X4,
            _ => Self::X2,
        }
    }
}

impl std::fmt::Display for LsfgMultiplier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x", self.value())
    }
}

/// Scaling type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ScalingType {
    #[default]
    None,
    Integer,
    /// Lossless Scaling 1 (LS1)
    Ls1,
    /// NVIDIA Image Scaling
    Nis,
    /// AMD FidelityFX Super Resolution
    Fsr,
}

impl std::fmt::Display for ScalingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "Off"),
            Self::Integer => write!(f, "Integer"),
            Self::Ls1 => write!(f, "LS1"),
            Self::Nis => write!(f, "NIS"),
            Self::Fsr => write!(f, "FSR"),
        }
    }
}

// ============================================================================
// Recommendations
// ============================================================================

/// Recommendation for Lossless Scaling settings
#[derive(Debug, Clone)]
pub struct LsRecommendation {
    pub should_use: bool,
    pub reason: String,
    pub profile: LsProfile,
}

/// Generate LSFG recommendation based on hardware and target
pub fn recommend_lsfg(
    native_fps: u32,
    target_fps: u32,
    has_native_frame_gen: bool,
) -> LsRecommendation {
    // Don't use LSFG if game has native frame generation
    if has_native_frame_gen {
        return LsRecommendation {
            should_use: false,
            reason: "Game has native frame generation (DLSS 3 / FSR 3)".to_string(),
            profile: LsProfile::default(),
        };
    }

    // Calculate best multiplier to reach target
    let ratios = [
        (LsfgMultiplier::X2, target_fps as f32 / 2.0),
        (LsfgMultiplier::X3, target_fps as f32 / 3.0),
        (LsfgMultiplier::X4, target_fps as f32 / 4.0),
    ];

    // Find multiplier where native FPS can meet the base requirement
    for (multiplier, base_needed) in ratios {
        if native_fps as f32 >= base_needed * 0.9 {
            // Check if LSFG is actually beneficial
            if native_fps < target_fps && base_needed >= 30.0 {
                let mut profile = LsProfile::default();
                profile.frame_generation = "LSFG3".to_string();
                profile.lsfg3_mode = "FIXED".to_string();
                profile.lsfg3_multiplier = multiplier.value();
                profile.lsfg3_target = target_fps;

                return LsRecommendation {
                    should_use: true,
                    reason: format!(
                        "Cap game at {} FPS, LSFG {} will output ~{} FPS",
                        base_needed.round() as u32,
                        multiplier,
                        (base_needed * multiplier.value() as f32).round() as u32
                    ),
                    profile,
                };
            }
        }
    }

    // Native FPS is good enough, no need for LSFG
    if native_fps >= target_fps {
        return LsRecommendation {
            should_use: false,
            reason: format!("Native FPS ({}) meets target ({})", native_fps, target_fps),
            profile: LsProfile::default(),
        };
    }

    // Native FPS too low for LSFG to help
    LsRecommendation {
        should_use: false,
        reason: format!(
            "Native FPS ({}) too low for effective LSFG (need at least 30-40)",
            native_fps
        ),
        profile: LsProfile::default(),
    }
}
