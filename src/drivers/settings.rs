//! NVIDIA driver setting definitions
//!
//! These correspond to settings in NVIDIA Control Panel's "Manage 3D Settings"

use serde::{Deserialize, Serialize};
use std::fmt;

/// Known NVIDIA driver settings that can be configured per-application
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NvidiaSetting {
    /// Maximum pre-rendered frames (Low Latency Mode)
    /// 0 = Use 3D application setting
    /// 1 = Ultra (minimum latency)
    /// 2-4 = Specific frame count
    PreRenderLimit,

    /// Frame Rate Limiter
    /// 0 = Off
    /// 1+ = Target FPS
    FrameRateLimit,

    /// Vertical Sync
    VSync,

    /// Anisotropic Filtering
    AnisotropicFiltering,

    /// Shader Cache
    ShaderCache,

    /// Threaded Optimization
    ThreadedOptimization,

    /// Power Management Mode
    PowerManagement,

    /// Texture Filtering Quality
    TextureFilteringQuality,

    /// Texture Filtering - Negative LOD Bias
    TextureFilteringLodBias,

    /// Texture Filtering - Trilinear Optimization
    TrilinearOptimization,

    /// Texture Filtering - Anisotropic Sample Optimization
    AnisotropicSampleOptimization,

    /// Anti-aliasing Mode
    AntiAliasingMode,

    /// Anti-aliasing Setting
    AntiAliasingSetting,

    /// CUDA - Force P2 State
    CudaForceP2State,

    /// G-SYNC indicator
    GSyncIndicator,

    /// Multi-Frame Sampled AA (MFAA)
    Mfaa,

    /// OpenGL rendering GPU
    OpenGlRenderingGpu,

    /// Preferred refresh rate
    PreferredRefreshRate,

    /// Triple buffering
    TripleBuffering,

    /// Maximum allowed FPS for background applications
    BackgroundMaxFps,
}

/// NVAPI DRS setting IDs, taken verbatim from NVIDIA's public `NvApiDriverSettings.h`
/// header (<https://github.com/NVIDIA/nvapi/blob/main/NvApiDriverSettings.h>).
pub mod nvapi_ids {
    // Performance / latency
    pub const PRERENDERLIMIT: u32 = 0x007BA09E;
    /// Modern framerate limiter (driver >= 441.x)
    pub const FRL_FPS: u32 = 0x10835002;
    pub const APPIDLE_DYNAMIC_FRL_FPS: u32 = 0x10835016;
    pub const PREFERRED_PSTATE: u32 = 0x1057EB71;

    // VSync
    pub const VSYNCMODE: u32 = 0x00A879CF;
    pub const VSYNC_TEAR_CONTROL: u32 = 0x005A375C;

    // OpenGL
    pub const OGL_THREAD_CONTROL: u32 = 0x20C1221E;
    pub const OGL_TRIPLE_BUFFER: u32 = 0x20FDD1F9;
    pub const OGL_IMPLICIT_GPU_AFFINITY: u32 = 0x20D0F3E6;

    // Shader cache
    pub const SHADER_DISK_CACHE: u32 = 0x00198FFF;

    // Anisotropic filtering
    pub const ANISO_MODE_LEVEL: u32 = 0x101E61A9;
    pub const ANISO_MODE_SELECTOR: u32 = 0x10D2BB16;

    // Anti-aliasing
    pub const AA_MODE_SELECTOR: u32 = 0x107EFC5B;
    pub const AA_MODE_METHOD: u32 = 0x10D773D2;
    pub const MAXWELL_B_SAMPLE_INTERLEAVE: u32 = 0x0098C1AC; // MFAA

    // Texture filtering
    pub const QUALITY_ENHANCEMENTS: u32 = 0x00CE2691;
    pub const LODBIASADJUST: u32 = 0x00738E8F;
    pub const PS_TEXFILTER_ANISO_OPTS2: u32 = 0x00E73211;
    pub const PS_TEXFILTER_BILINEAR_IN_ANISO: u32 = 0x0084CD70;
    pub const PS_TEXFILTER_DISABLE_TRILIN_SLOPE: u32 = 0x002ECAF2;
    pub const PS_TEXFILTER_NO_NEG_LODBIAS: u32 = 0x0019BB68;

    // Display
    pub const REFRESH_RATE_OVERRIDE: u32 = 0x0064B541;
    pub const VRR_FEATURE_INDICATOR: u32 = 0x1094F157;
    pub const VRR_OVERLAY_INDICATOR: u32 = 0x1095F16F;

    // CUDA
    /// Not published in NVIDIA's public NVAPI header; sourced from community tools
    /// (Nvidia Profile Inspector). Works on current drivers but may be removed in future SDKs.
    pub const CUDA_FORCE_P2_STATE: u32 = 0x5072FE85;
}

/// VSYNCMODE values (verbatim from the public NVAPI header).
pub mod vsync_values {
    /// Use 3D application setting
    pub const PASSIVE: u32 = 0x60925292;
    /// Force off
    pub const FORCEOFF: u32 = 0x08416747;
    /// Force on
    pub const FORCEON: u32 = 0x47814940;
}

/// VSYNCTEARCONTROL values (for adaptive vsync together with FORCEON).
pub mod vsync_tear_values {
    pub const DISABLE: u32 = 0x96861077;
    pub const ENABLE: u32 = 0x99941284;
}

/// OGL_THREAD_CONTROL values
pub mod thread_control_values {
    pub const DEFAULT: u32 = 0x00000000;
    pub const ENABLE: u32 = 0x00000001;
    pub const DISABLE: u32 = 0x00000002;
}

/// SHADER_DISK_CACHE values
pub mod shader_cache_values {
    pub const OFF: u32 = 0x00000000;
    pub const ON: u32 = 0x00000001;
}

/// PREFERRED_PSTATE values
pub mod pstate_values {
    pub const ADAPTIVE: u32 = 0x00000000;
    pub const PREFER_MAX_PERFORMANCE: u32 = 0x00000001;
    pub const DRIVER_CONTROLLED: u32 = 0x00000002;
    pub const PREFER_CONSISTENT_PERFORMANCE: u32 = 0x00000003;
    pub const PREFER_MIN: u32 = 0x00000004;
    pub const OPTIMAL_POWER: u32 = 0x00000005;
}

/// ANISO_MODE_SELECTOR values (application-control vs. user override / enhance).
pub mod aniso_selector_values {
    pub const APP_CONTROL: u32 = 0x00000000;
    pub const USER_DEFINED: u32 = 0x00000001;
    pub const CONDITIONAL: u32 = 0x00000002;
}

/// ANISO_MODE_LEVEL values (1 = app controlled / off; other values are raw 2,4,8,16x).
pub mod aniso_level_values {
    pub const NONE: u32 = 0x00000001;
    pub const X2: u32 = 0x00000002;
    pub const X4: u32 = 0x00000004;
    pub const X8: u32 = 0x00000008;
    pub const X16: u32 = 0x00000010;
}

/// OGL_TRIPLE_BUFFER values
pub mod triple_buffer_values {
    pub const DISABLED: u32 = 0x00000000;
    pub const ENABLED: u32 = 0x00000001;
}

/// MAXWELL_B_SAMPLE_INTERLEAVE (MFAA) values
pub mod mfaa_values {
    pub const OFF: u32 = 0x00000000;
    pub const ON: u32 = 0x00000001;
}

/// QUALITY_ENHANCEMENTS (Texture filtering - Quality) values
pub mod quality_enhancements_values {
    pub const HIGH_QUALITY: u32 = 0xFFFFFFF6; // -10 as i32
    pub const QUALITY: u32 = 0x00000000;
    pub const PERFORMANCE: u32 = 0x0000000A;
    pub const HIGH_PERFORMANCE: u32 = 0x00000014;
}

impl NvidiaSetting {
    /// Get the NVAPI setting ID string
    pub fn nvapi_id(&self) -> &'static str {
        match self {
            Self::PreRenderLimit => "PRERENDERLIMIT",
            Self::FrameRateLimit => "PS_FRAMERATE_LIMITER",
            Self::VSync => "VSYNCMODE",
            Self::AnisotropicFiltering => "PS_TEXFILTER_ANISO_OPTS2",
            Self::ShaderCache => "PS_SHADERDISKCACHE",
            Self::ThreadedOptimization => "OGL_THREAD_CONTROL",
            Self::PowerManagement => "PREFERRED_PSTATE",
            Self::TextureFilteringQuality => "QUALITY_ENHANCEMENTS",
            Self::TextureFilteringLodBias => "LODBIASADJUST",
            Self::TrilinearOptimization => "PS_TEXFILTER_BILINEAR_IN_ANISO",
            Self::AnisotropicSampleOptimization => "PS_TEXFILTER_NO_NEG_LODBIAS",
            Self::AntiAliasingMode => "ANTIALIASING_MODE",
            Self::AntiAliasingSetting => "ANTIALIASING_SETTING",
            Self::CudaForceP2State => "CUDA_FORCE_P2_STATE",
            Self::GSyncIndicator => "GSYNC_INDICATOR",
            Self::Mfaa => "MAXWELL_B_SAMPLE_INTERLEAVE",
            Self::OpenGlRenderingGpu => "OGL_DEFAULT_RENDERING_GPU",
            Self::PreferredRefreshRate => "PREFERRED_REFRESH_RATE",
            Self::TripleBuffering => "OGL_FORCE_TRIPLE_BUFFER",
            Self::BackgroundMaxFps => "PS_FRAMERATE_LIMITER_GPS",
        }
    }

    /// Numeric NVAPI DRS setting ID (the value passed to `NvAPI_DRS_GetSetting`/`SetSetting`).
    ///
    /// All IDs come from NVIDIA's public `NvApiDriverSettings.h` header, except
    /// `CudaForceP2State` which is community-sourced (see [`nvapi_ids::CUDA_FORCE_P2_STATE`]).
    pub fn setting_id(&self) -> u32 {
        match self {
            Self::PreRenderLimit => nvapi_ids::PRERENDERLIMIT,
            Self::FrameRateLimit => nvapi_ids::FRL_FPS,
            Self::VSync => nvapi_ids::VSYNCMODE,
            Self::ShaderCache => nvapi_ids::SHADER_DISK_CACHE,
            Self::ThreadedOptimization => nvapi_ids::OGL_THREAD_CONTROL,
            Self::PowerManagement => nvapi_ids::PREFERRED_PSTATE,
            Self::CudaForceP2State => nvapi_ids::CUDA_FORCE_P2_STATE,
            Self::AnisotropicFiltering => nvapi_ids::ANISO_MODE_LEVEL,
            Self::TextureFilteringQuality => nvapi_ids::QUALITY_ENHANCEMENTS,
            Self::TextureFilteringLodBias => nvapi_ids::LODBIASADJUST,
            Self::TrilinearOptimization => nvapi_ids::PS_TEXFILTER_DISABLE_TRILIN_SLOPE,
            Self::AnisotropicSampleOptimization => nvapi_ids::PS_TEXFILTER_ANISO_OPTS2,
            Self::AntiAliasingMode => nvapi_ids::AA_MODE_SELECTOR,
            Self::AntiAliasingSetting => nvapi_ids::AA_MODE_METHOD,
            Self::GSyncIndicator => nvapi_ids::VRR_OVERLAY_INDICATOR,
            Self::Mfaa => nvapi_ids::MAXWELL_B_SAMPLE_INTERLEAVE,
            Self::OpenGlRenderingGpu => nvapi_ids::OGL_IMPLICIT_GPU_AFFINITY,
            Self::PreferredRefreshRate => nvapi_ids::REFRESH_RATE_OVERRIDE,
            Self::TripleBuffering => nvapi_ids::OGL_TRIPLE_BUFFER,
            Self::BackgroundMaxFps => nvapi_ids::APPIDLE_DYNAMIC_FRL_FPS,
        }
    }

    /// Get human-readable name for display
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::PreRenderLimit => "Low Latency Mode",
            Self::FrameRateLimit => "Max Frame Rate",
            Self::VSync => "Vertical Sync",
            Self::AnisotropicFiltering => "Anisotropic Filtering",
            Self::ShaderCache => "Shader Cache",
            Self::ThreadedOptimization => "Threaded Optimization",
            Self::PowerManagement => "Power Management Mode",
            Self::TextureFilteringQuality => "Texture Filtering - Quality",
            Self::TextureFilteringLodBias => "Texture Filtering - LOD Bias",
            Self::TrilinearOptimization => "Trilinear Optimization",
            Self::AnisotropicSampleOptimization => "Anisotropic Sample Optimization",
            Self::AntiAliasingMode => "Antialiasing - Mode",
            Self::AntiAliasingSetting => "Antialiasing - Setting",
            Self::CudaForceP2State => "CUDA Force P2 State",
            Self::GSyncIndicator => "G-SYNC Indicator",
            Self::Mfaa => "Multi-Frame Sampled AA (MFAA)",
            Self::OpenGlRenderingGpu => "OpenGL Rendering GPU",
            Self::PreferredRefreshRate => "Preferred Refresh Rate",
            Self::TripleBuffering => "Triple Buffering",
            Self::BackgroundMaxFps => "Background Application Max Frame Rate",
        }
    }
}

impl fmt::Display for NvidiaSetting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// VSync mode values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VSyncMode {
    /// Use 3D application setting
    ApplicationControlled,
    /// Force off
    Off,
    /// Force on
    On,
    /// Adaptive (half refresh rate)
    Adaptive,
    /// Fast sync
    Fast,
}

impl VSyncMode {
    /// Logical value (0..=4) used for user-facing serialization.
    pub fn to_value(&self) -> u32 {
        match self {
            Self::ApplicationControlled => 0,
            Self::Off => 1,
            Self::On => 2,
            Self::Adaptive => 3,
            Self::Fast => 4,
        }
    }

    pub fn from_value(v: u32) -> Option<Self> {
        match v {
            0 => Some(Self::ApplicationControlled),
            1 => Some(Self::Off),
            2 => Some(Self::On),
            3 => Some(Self::Adaptive),
            4 => Some(Self::Fast),
            _ => None,
        }
    }

    /// Raw NVAPI DRS DWORD value for the `VSYNCMODE` setting.
    ///
    /// Note: "Adaptive" is expressed in NVAPI as `FORCEON` combined with
    /// `VSYNC_TEAR_CONTROL = ENABLE` (see [`VSyncMode::tear_control_value`]).
    ///
    /// `Fast` sync no longer has a standalone `VSYNCMODE` value in the current
    /// public NVAPI header; this falls back to `FORCEON`, which is the closest
    /// equivalent. True Fast sync behaviour requires additional flags outside
    /// the scope of this mapping.
    pub fn to_nvapi_value(&self) -> u32 {
        match self {
            Self::ApplicationControlled => vsync_values::PASSIVE,
            Self::Off => vsync_values::FORCEOFF,
            Self::On | Self::Adaptive | Self::Fast => vsync_values::FORCEON,
        }
    }

    /// Returns the `VSYNC_TEAR_CONTROL` value to write alongside `VSYNCMODE`,
    /// or `None` when no tear-control write is needed.
    pub fn tear_control_value(&self) -> Option<u32> {
        match self {
            Self::Adaptive => Some(vsync_tear_values::ENABLE),
            Self::On | Self::Off | Self::Fast => Some(vsync_tear_values::DISABLE),
            Self::ApplicationControlled => None,
        }
    }

    /// Decode a raw NVAPI VSYNCMODE DWORD into a VSyncMode.
    pub fn from_nvapi_value(v: u32) -> Option<Self> {
        match v {
            x if x == vsync_values::PASSIVE => Some(Self::ApplicationControlled),
            x if x == vsync_values::FORCEOFF => Some(Self::Off),
            x if x == vsync_values::FORCEON => Some(Self::On),
            _ => None,
        }
    }
}

/// Power management mode values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PowerManagementMode {
    /// Adaptive
    Adaptive,
    /// Prefer maximum performance
    PreferMaxPerformance,
    /// Optimal power (default)
    Optimal,
}

impl PowerManagementMode {
    pub fn to_value(&self) -> u32 {
        match self {
            Self::Adaptive => 0,
            Self::PreferMaxPerformance => 1,
            Self::Optimal => 2,
        }
    }

    /// Raw NVAPI DRS DWORD value for the `PREFERRED_PSTATE` setting.
    pub fn to_nvapi_value(&self) -> u32 {
        match self {
            Self::Adaptive => pstate_values::ADAPTIVE,
            Self::PreferMaxPerformance => pstate_values::PREFER_MAX_PERFORMANCE,
            Self::Optimal => pstate_values::DRIVER_CONTROLLED,
        }
    }

    pub fn from_nvapi_value(v: u32) -> Option<Self> {
        match v {
            pstate_values::ADAPTIVE => Some(Self::Adaptive),
            pstate_values::PREFER_MAX_PERFORMANCE => Some(Self::PreferMaxPerformance),
            pstate_values::DRIVER_CONTROLLED
            | pstate_values::PREFER_CONSISTENT_PERFORMANCE
            | pstate_values::OPTIMAL_POWER => Some(Self::Optimal),
            _ => None,
        }
    }
}

/// Texture filtering - Quality level (maps to `QUALITY_ENHANCEMENTS` DRS setting).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextureFilterQuality {
    HighQuality,
    Quality,
    Performance,
    HighPerformance,
}

impl TextureFilterQuality {
    pub fn to_nvapi_value(&self) -> u32 {
        match self {
            Self::HighQuality => quality_enhancements_values::HIGH_QUALITY,
            Self::Quality => quality_enhancements_values::QUALITY,
            Self::Performance => quality_enhancements_values::PERFORMANCE,
            Self::HighPerformance => quality_enhancements_values::HIGH_PERFORMANCE,
        }
    }

    pub fn from_nvapi_value(v: u32) -> Option<Self> {
        match v {
            quality_enhancements_values::HIGH_QUALITY => Some(Self::HighQuality),
            quality_enhancements_values::QUALITY => Some(Self::Quality),
            quality_enhancements_values::PERFORMANCE => Some(Self::Performance),
            quality_enhancements_values::HIGH_PERFORMANCE => Some(Self::HighPerformance),
            _ => None,
        }
    }

    /// Parse from case-insensitive names used by the AI response and CLI flags.
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().replace('-', "_").as_str() {
            "high_quality" | "highquality" | "hq" => Some(Self::HighQuality),
            "quality" => Some(Self::Quality),
            "performance" | "perf" => Some(Self::Performance),
            "high_performance" | "highperformance" | "hp" => Some(Self::HighPerformance),
            _ => None,
        }
    }
}

/// Anisotropic filtering level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnisotropicLevel {
    ApplicationControlled,
    X2,
    X4,
    X8,
    X16,
}

impl AnisotropicLevel {
    pub fn to_value(&self) -> u32 {
        match self {
            Self::ApplicationControlled => 0,
            Self::X2 => 2,
            Self::X4 => 4,
            Self::X8 => 8,
            Self::X16 => 16,
        }
    }

    /// Raw NVAPI DRS DWORD value for `ANISO_MODE_LEVEL`.
    pub fn to_nvapi_value(&self) -> u32 {
        match self {
            Self::ApplicationControlled => aniso_level_values::NONE,
            Self::X2 => aniso_level_values::X2,
            Self::X4 => aniso_level_values::X4,
            Self::X8 => aniso_level_values::X8,
            Self::X16 => aniso_level_values::X16,
        }
    }

    /// Value to write for `ANISO_MODE_SELECTOR` alongside the level.
    pub fn to_selector_value(&self) -> u32 {
        match self {
            Self::ApplicationControlled => aniso_selector_values::APP_CONTROL,
            _ => aniso_selector_values::USER_DEFINED,
        }
    }

    pub fn from_nvapi_value(v: u32) -> Option<Self> {
        match v {
            aniso_level_values::NONE => Some(Self::ApplicationControlled),
            aniso_level_values::X2 => Some(Self::X2),
            aniso_level_values::X4 => Some(Self::X4),
            aniso_level_values::X8 => Some(Self::X8),
            aniso_level_values::X16 => Some(Self::X16),
            _ => None,
        }
    }
}

/// A complete set of recommended driver settings for a game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverProfile {
    /// Profile name (usually game executable name)
    pub name: String,

    /// Associated executable(s)
    pub executables: Vec<String>,

    /// Low latency mode (pre-render limit)
    pub low_latency: Option<u32>,

    /// Frame rate limit (0 = off)
    pub frame_rate_limit: Option<u32>,

    /// VSync mode
    pub vsync: Option<VSyncMode>,

    /// Power management mode
    pub power_management: Option<PowerManagementMode>,

    /// Anisotropic filtering level
    pub anisotropic_filtering: Option<AnisotropicLevel>,

    /// Shader cache enabled
    pub shader_cache: Option<bool>,

    /// Threaded optimization enabled
    pub threaded_optimization: Option<bool>,

    /// MFAA enabled
    pub mfaa: Option<bool>,

    /// Triple buffering enabled
    pub triple_buffering: Option<bool>,

    /// Texture Filtering - Quality
    #[serde(default)]
    pub texture_filter_quality: Option<TextureFilterQuality>,

    /// Preferred refresh rate: Some(true) = Highest Available, Some(false) = App Controlled
    #[serde(default)]
    pub preferred_refresh_rate: Option<bool>,

    /// Background application FPS cap (0 = off)
    #[serde(default)]
    pub background_fps_limit: Option<u32>,
}

impl Default for DriverProfile {
    fn default() -> Self {
        Self {
            name: String::new(),
            executables: Vec::new(),
            low_latency: None,
            frame_rate_limit: None,
            vsync: None,
            power_management: None,
            anisotropic_filtering: None,
            shader_cache: None,
            threaded_optimization: None,
            mfaa: None,
            triple_buffering: None,
            texture_filter_quality: None,
            preferred_refresh_rate: None,
            background_fps_limit: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vsync_mode_round_trips_through_nvapi_values() {
        for mode in [
            VSyncMode::ApplicationControlled,
            VSyncMode::Off,
            VSyncMode::On,
        ] {
            assert_eq!(VSyncMode::from_nvapi_value(mode.to_nvapi_value()), Some(mode));
        }
        // Adaptive and Fast both encode to FORCEON, which decodes back to On.
        assert_eq!(
            VSyncMode::from_nvapi_value(VSyncMode::Adaptive.to_nvapi_value()),
            Some(VSyncMode::On)
        );
    }

    #[test]
    fn vsync_tear_control_pairings() {
        assert_eq!(VSyncMode::Adaptive.tear_control_value(), Some(vsync_tear_values::ENABLE));
        assert_eq!(VSyncMode::On.tear_control_value(), Some(vsync_tear_values::DISABLE));
        assert_eq!(VSyncMode::Off.tear_control_value(), Some(vsync_tear_values::DISABLE));
        assert_eq!(VSyncMode::ApplicationControlled.tear_control_value(), None);
    }

    #[test]
    fn power_management_round_trips() {
        for mode in [
            PowerManagementMode::Adaptive,
            PowerManagementMode::PreferMaxPerformance,
            PowerManagementMode::Optimal,
        ] {
            assert_eq!(
                PowerManagementMode::from_nvapi_value(mode.to_nvapi_value()),
                Some(mode)
            );
        }
        // OPTIMAL_POWER and PREFER_CONSISTENT_PERFORMANCE both map back to Optimal.
        assert_eq!(
            PowerManagementMode::from_nvapi_value(pstate_values::OPTIMAL_POWER),
            Some(PowerManagementMode::Optimal)
        );
        assert_eq!(
            PowerManagementMode::from_nvapi_value(pstate_values::PREFER_CONSISTENT_PERFORMANCE),
            Some(PowerManagementMode::Optimal)
        );
    }

    #[test]
    fn anisotropic_level_round_trips() {
        for level in [
            AnisotropicLevel::ApplicationControlled,
            AnisotropicLevel::X2,
            AnisotropicLevel::X4,
            AnisotropicLevel::X8,
            AnisotropicLevel::X16,
        ] {
            assert_eq!(
                AnisotropicLevel::from_nvapi_value(level.to_nvapi_value()),
                Some(level)
            );
        }
    }

    #[test]
    fn anisotropic_selector_uses_user_defined_for_override() {
        assert_eq!(
            AnisotropicLevel::X16.to_selector_value(),
            aniso_selector_values::USER_DEFINED
        );
        assert_eq!(
            AnisotropicLevel::ApplicationControlled.to_selector_value(),
            aniso_selector_values::APP_CONTROL
        );
    }

    #[test]
    fn texture_filter_quality_round_trips() {
        for q in [
            TextureFilterQuality::HighQuality,
            TextureFilterQuality::Quality,
            TextureFilterQuality::Performance,
            TextureFilterQuality::HighPerformance,
        ] {
            assert_eq!(
                TextureFilterQuality::from_nvapi_value(q.to_nvapi_value()),
                Some(q)
            );
        }
    }

    #[test]
    fn texture_filter_quality_parses_names() {
        assert_eq!(
            TextureFilterQuality::from_name("high_quality"),
            Some(TextureFilterQuality::HighQuality)
        );
        assert_eq!(
            TextureFilterQuality::from_name("HIGH-PERFORMANCE"),
            Some(TextureFilterQuality::HighPerformance)
        );
        assert_eq!(
            TextureFilterQuality::from_name("perf"),
            Some(TextureFilterQuality::Performance)
        );
        assert_eq!(TextureFilterQuality::from_name("bogus"), None);
    }

    #[test]
    fn setting_id_is_stable_for_every_variant() {
        // Guards against accidental removal; every variant must return an ID.
        for s in [
            NvidiaSetting::PreRenderLimit,
            NvidiaSetting::FrameRateLimit,
            NvidiaSetting::VSync,
            NvidiaSetting::AnisotropicFiltering,
            NvidiaSetting::ShaderCache,
            NvidiaSetting::ThreadedOptimization,
            NvidiaSetting::PowerManagement,
            NvidiaSetting::TextureFilteringQuality,
            NvidiaSetting::TextureFilteringLodBias,
            NvidiaSetting::TrilinearOptimization,
            NvidiaSetting::AnisotropicSampleOptimization,
            NvidiaSetting::AntiAliasingMode,
            NvidiaSetting::AntiAliasingSetting,
            NvidiaSetting::CudaForceP2State,
            NvidiaSetting::GSyncIndicator,
            NvidiaSetting::Mfaa,
            NvidiaSetting::OpenGlRenderingGpu,
            NvidiaSetting::PreferredRefreshRate,
            NvidiaSetting::TripleBuffering,
            NvidiaSetting::BackgroundMaxFps,
        ] {
            // Non-zero, non-sentinel DWORD
            let id = s.setting_id();
            assert_ne!(id, 0, "{} returned 0 ID", s.display_name());
            assert_ne!(id, u32::MAX, "{} returned sentinel ID", s.display_name());
        }
    }

    #[test]
    fn known_vsync_ids_match_public_header() {
        // Sanity against the values we verified against NvApiDriverSettings.h.
        assert_eq!(nvapi_ids::VSYNCMODE, 0x00A879CF);
        assert_eq!(nvapi_ids::VSYNC_TEAR_CONTROL, 0x005A375C);
        assert_eq!(vsync_values::PASSIVE, 0x60925292);
        assert_eq!(vsync_values::FORCEOFF, 0x08416747);
        assert_eq!(vsync_values::FORCEON, 0x47814940);
        assert_eq!(vsync_tear_values::DISABLE, 0x96861077);
        assert_eq!(vsync_tear_values::ENABLE, 0x99941284);
    }
}
