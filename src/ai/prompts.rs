//! Prompt templates and data structures for AI recommendations

use serde::{Deserialize, Serialize};

use crate::data::pcgamingwiki::GameInfo;
use crate::data::benchmarks::RecommendedSettings;
use crate::hardware::gpu::GpuInfo;

/// Request for game optimization recommendations
#[derive(Debug, Clone, Serialize)]
pub struct RecommendationRequest {
    /// Game name
    pub game_name: String,
    /// GPU information
    pub gpu: GpuInfo,
    /// Target resolution (e.g., "2560x1440")
    pub target_resolution: String,
    /// Target framerate
    pub target_fps: u32,
    /// Whether G-Sync/FreeSync is available
    pub vrr_enabled: bool,
    /// PCGamingWiki data if available
    pub pcgw_data: Option<GameInfo>,
    /// Benchmark data if available
    pub benchmark_data: Option<RecommendedSettings>,
    /// Whether to only recommend driver settings
    pub driver_only: bool,
}

/// AI-generated optimization recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationResponse {
    /// Game name
    pub game: String,
    /// Summary of recommendations
    pub summary: String,
    /// NVIDIA driver profile settings
    pub driver_settings: DriverSettings,
    /// In-game graphics settings (if requested)
    #[serde(default)]
    pub in_game_settings: Option<InGameSettings>,
    /// Lossless Scaling recommendations (if beneficial)
    #[serde(default)]
    pub lossless_scaling: Option<LosslessScalingSettings>,
    /// RTSS framerate limiter recommendation
    #[serde(default)]
    pub rtss_fps_limit: Option<u32>,
    /// Reasoning for each major recommendation
    pub reasoning: Vec<String>,
    /// Potential issues or caveats
    #[serde(default)]
    pub caveats: Vec<String>,
}

/// NVIDIA Control Panel driver settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DriverSettings {
    /// Frame rate limit (0 = off)
    #[serde(default)]
    pub frame_rate_limit: Option<u32>,
    /// Low latency mode: 0=off, 1=on, 2=ultra
    #[serde(default)]
    pub low_latency_mode: Option<u32>,
    /// VSync mode
    #[serde(default)]
    pub vsync: Option<String>,
    /// Power management mode
    #[serde(default)]
    pub power_management: Option<String>,
    /// Texture filtering quality
    #[serde(default)]
    pub texture_filtering: Option<String>,
    /// Anisotropic filtering level
    #[serde(default)]
    pub anisotropic_filtering: Option<u32>,
    /// Threaded optimization
    #[serde(default)]
    pub threaded_optimization: Option<String>,
    /// Shader cache
    #[serde(default)]
    pub shader_cache: Option<String>,
    /// CUDA - Force P2 State
    #[serde(default)]
    pub cuda_force_p2: Option<bool>,
    /// Image sharpening
    #[serde(default)]
    pub image_sharpening: Option<String>,
}

/// In-game graphics settings recommendations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InGameSettings {
    /// Overall preset recommendation
    #[serde(default)]
    pub preset: Option<String>,
    /// Resolution scale / render resolution
    #[serde(default)]
    pub resolution_scale: Option<String>,
    /// Ray tracing recommendation
    #[serde(default)]
    pub ray_tracing: Option<String>,
    /// DLSS / FSR / XeSS recommendation
    #[serde(default)]
    pub upscaling: Option<String>,
    /// Frame generation (DLSS 3 / FSR 3)
    #[serde(default)]
    pub frame_generation: Option<String>,
    /// Individual settings to adjust from preset
    #[serde(default)]
    pub adjustments: Vec<SettingAdjustment>,
}

/// A single setting adjustment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingAdjustment {
    pub setting: String,
    pub value: String,
    pub reason: Option<String>,
}

/// Lossless Scaling settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LosslessScalingSettings {
    /// Whether to use Lossless Scaling
    pub enabled: bool,
    /// LSFG mode (e.g., "LSFG 2.3")
    #[serde(default)]
    pub mode: Option<String>,
    /// Frame generation multiplier
    #[serde(default)]
    pub multiplier: Option<String>,
    /// Base framerate to cap game at
    #[serde(default)]
    pub base_fps: Option<u32>,
    /// Scaling type (e.g., "Integer", "NIS", "FSR")
    #[serde(default)]
    pub scaling_type: Option<String>,
}

impl RecommendationRequest {
    /// Build the system prompt for Claude
    pub fn system_prompt(&self) -> String {
        format!(
            r#"You are an expert PC gaming optimization specialist. Your role is to analyze hardware specifications and game characteristics to recommend optimal settings for the best gaming experience.

You must respond with a JSON object matching this exact schema:
{{
  "game": "string",
  "summary": "Brief 1-2 sentence summary",
  "driver_settings": {{
    "frame_rate_limit": number or null,
    "low_latency_mode": 0|1|2 or null,
    "vsync": "off"|"on"|"adaptive"|"fast" or null,
    "power_management": "adaptive"|"prefer_max_performance"|"optimal" or null,
    "texture_filtering": "quality"|"performance"|"high_performance" or null,
    "anisotropic_filtering": 2|4|8|16 or null,
    "threaded_optimization": "auto"|"on"|"off" or null,
    "shader_cache": "on"|"off"|"unlimited" or null
  }},
  "in_game_settings": {{
    "preset": "Low"|"Medium"|"High"|"Ultra"|"Custom" or null,
    "resolution_scale": "Native"|"DLSS Quality"|"DLSS Balanced"|"FSR Quality" etc or null,
    "ray_tracing": "Off"|"Low"|"Medium"|"High"|"Ultra"|"Psycho" or null,
    "upscaling": "DLSS"|"FSR"|"XeSS"|"Native" with quality mode or null,
    "frame_generation": "On"|"Off" or null,
    "adjustments": [{{"setting": "name", "value": "recommended", "reason": "why"}}]
  }},
  "lossless_scaling": {{
    "enabled": true if LSFG would benefit this setup,
    "mode": "LSFG 2.3" or other version,
    "multiplier": "2x"|"3x"|"4x",
    "base_fps": recommended FPS cap for the game when using LSFG,
    "scaling_type": "None"|"Integer"|"NIS"|"FSR" or null
  }},
  "rtss_fps_limit": number or null,
  "reasoning": ["string"],
  "caveats": ["string"]
}}

CRITICAL RULES:
1. ONLY recommend in-game features that ACTUALLY EXIST in this specific game
2. If PCGamingWiki shows ray_tracing as "Unknown" or "No", do NOT recommend ray tracing settings
3. If frame generation is not a native feature of this game, set frame_generation to null
4. Be REALISTIC about performance - triple 4K (7680x2160) is extremely demanding, expect 30-60 FPS in AAA games even on high-end GPUs
5. Use PCGamingWiki data to determine what features the game actually supports
6. When uncertain about a feature, omit it rather than guess
7. For Lossless Scaling: only recommend if native FPS would be stable at 40-60 and target is 120+Hz

Key optimization principles:
1. For VRR (G-Sync/FreeSync) displays: Cap FPS 2-3 below refresh rate to stay in VRR range
2. Low Latency Mode Ultra is best for competitive games, but can cause stuttering in poorly optimized games
3. RTSS provides more consistent frame pacing than in-game or driver limiters
4. Lossless Scaling frame generation is most beneficial when base FPS is 40-60 and target is 120+
5. Ray tracing has significant performance cost - only recommend if GPU has ample headroom
6. DLSS/FSR Quality mode is usually the best balance of performance and visuals
7. Power Management "Prefer Maximum Performance" is only needed for inconsistent framerates

Hardware being optimized for:
- GPU: {} ({} VRAM)
- Driver: {}
- Target: {} @ {} FPS
- VRR: {}"#,
            self.gpu.name,
            self.gpu.vram_display,
            self.gpu.driver_version,
            self.target_resolution,
            self.target_fps,
            if self.vrr_enabled { "Enabled" } else { "Disabled" }
        )
    }

    /// Build the user message with all context
    pub fn user_message(&self) -> String {
        let mut msg = format!(
            "Please recommend optimal settings for: {}\n\n",
            self.game_name
        );

        // Add PCGamingWiki data if available
        if let Some(ref pcgw) = self.pcgw_data {
            msg.push_str("## PCGamingWiki Data\n");
            msg.push_str(&format!("- Developers: {}\n", pcgw.developers.join(", ")));
            msg.push_str(&format!("- Engine: {}\n", pcgw.engines.join(", ")));
            
            if let Some(ref video) = pcgw.video {
                msg.push_str(&format!("- Widescreen: {}\n", video.widescreen));
                msg.push_str(&format!("- 4K Support: {}\n", video.four_k));
                msg.push_str(&format!("- HDR: {}\n", video.hdr));
                msg.push_str(&format!("- Ray Tracing: {}\n", video.ray_tracing));
                msg.push_str(&format!("- Unlocked FPS: {}\n", video.fps_unlimited));
            }
            msg.push('\n');
        }

        // Add benchmark data if available
        if let Some(ref bench) = self.benchmark_data {
            msg.push_str("## Benchmark Reference Data\n");
            if let Some(ref preset) = bench.preset {
                msg.push_str(&format!("- Recommended Preset: {}\n", preset));
            }
            for setting in &bench.settings {
                msg.push_str(&format!("- {}: {}", setting.name, setting.value));
                if let Some(ref impact) = setting.impact {
                    msg.push_str(&format!(" ({})", impact));
                }
                msg.push('\n');
            }
            for note in &bench.notes {
                msg.push_str(&format!("- Note: {}\n", note));
            }
            msg.push('\n');
        }

        if self.driver_only {
            msg.push_str("\nFocus only on NVIDIA driver profile settings. Do not include in-game settings.\n");
        }

        msg.push_str("\nProvide your recommendations as a JSON object.");

        msg
    }
}

/// Quick/heuristic-based recommendations without AI
pub fn quick_recommend(gpu: &GpuInfo, target_fps: u32, vrr_enabled: bool) -> DriverSettings {
    let mut settings = DriverSettings::default();

    // Frame rate limit: 2-3 below target for VRR, exact for non-VRR
    settings.frame_rate_limit = Some(if vrr_enabled {
        target_fps.saturating_sub(3)
    } else {
        target_fps
    });

    // Low latency: On for most games (Ultra can cause issues)
    settings.low_latency_mode = Some(1);

    // VSync: Off with VRR, Fast Sync without
    settings.vsync = Some(if vrr_enabled {
        "off".to_string()
    } else {
        "fast".to_string()
    });

    // Power management: Optimal unless high-end GPU
    let vram_gb = gpu.vram_bytes / (1024 * 1024 * 1024);
    settings.power_management = Some(if vram_gb >= 12 {
        "prefer_max_performance".to_string()
    } else {
        "optimal".to_string()
    });

    // Texture filtering: Quality for most GPUs
    settings.texture_filtering = Some("quality".to_string());

    // Anisotropic: 16x is essentially free on modern GPUs
    settings.anisotropic_filtering = Some(16);

    // Threaded optimization: Auto
    settings.threaded_optimization = Some("auto".to_string());

    // Shader cache: Unlimited if available
    settings.shader_cache = Some("unlimited".to_string());

    settings
}
