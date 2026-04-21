//! NVIDIA driver profile management via NVAPI
//!
//! Uses the DRS (Driver Settings) API to enumerate and modify application profiles.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::drs::{DrsSession, NvDRSProfileHandle, status_message, wstring_to_string};
use super::settings::{
    AnisotropicLevel, DriverProfile, NvidiaSetting, PowerManagementMode, TextureFilterQuality,
    VSyncMode, mfaa_values, nvapi_ids, shader_cache_values, thread_control_values,
    triple_buffer_values,
};

/// Represents an NVIDIA application profile in the driver
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NvidiaProfile {
    /// Profile name
    pub name: String,

    /// Associated application executables
    pub applications: Vec<String>,

    /// Profile settings as key-value pairs
    pub settings: HashMap<String, ProfileSettingValue>,

    /// Whether this is a predefined (NVIDIA) profile or user-created
    pub is_predefined: bool,
}

/// A profile setting value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSettingValue {
    /// Setting ID
    pub id: String,

    /// Current value (as DWORD)
    pub value: u32,

    /// Human-readable value description
    pub display_value: Option<String>,
}

/// NVIDIA profile manager for reading/writing driver profiles
pub struct NvidiaProfileManager {
    session: Option<DrsSession>,
}

impl NvidiaProfileManager {
    /// Create a new profile manager
    pub fn new() -> Result<Self> {
        // Initialize NVAPI
        nvapi::initialize().context("Failed to initialize NVAPI")?;
        
        // Try to create DRS session
        let session = DrsSession::new();
        if session.is_none() {
            tracing::warn!("Failed to create DRS session - profile enumeration limited");
        }
        
        Ok(Self { session })
    }

    /// List all application profiles (defaults to user-modified only)
    pub fn list_profiles(&self) -> Result<Vec<NvidiaProfile>> {
        self.list_profiles_filtered(false, None)
    }

    /// Borrow the underlying DRS session (None if it could not be created).
    pub fn session(&self) -> Option<&DrsSession> {
        self.session.as_ref()
    }

    /// List profiles with filtering options
    ///
    /// When `show_all=false`, shows profiles with applications (matches NVCP Program Settings)
    /// Use `show_all=true` to include all profiles including those without apps
    pub fn list_profiles_filtered(&self, show_all: bool, filter: Option<&str>) -> Result<Vec<NvidiaProfile>> {
        let session = match &self.session {
            Some(s) => s,
            None => {
                // Fallback to base profile only
                return Ok(vec![NvidiaProfile {
                    name: "Base Profile".to_string(),
                    applications: vec!["*".to_string()],
                    settings: HashMap::new(),
                    is_predefined: true,
                }]);
            }
        };
        
        let mut profiles = Vec::new();
        let filter_lower = filter.map(|f| f.to_lowercase());
        
        for (handle, info) in session.enumerate_profiles() {
            let is_predefined = info.is_predefined != 0;
            let has_apps = info.num_of_apps > 0;
            
            // Debug: log profile info
            let name = wstring_to_string(&info.profile_name);
            tracing::trace!(
                "Profile '{}': predefined={}, apps={}, settings={}",
                name, is_predefined, info.num_of_apps, info.num_of_settings
            );
            
            // Filter logic:
            // - If show_all, include everything (even profiles without apps)
            // - Otherwise, only show profiles that have applications assigned
            //   This matches what users see in NVCP's "Program Settings" dropdown
            if !show_all && !has_apps {
                continue;
            }
            
            // Apply name filter if provided
            if let Some(ref f) = filter_lower {
                if !name.to_lowercase().contains(f) {
                    continue;
                }
            }
            
            // Get applications for this profile
            let apps = session.get_applications(handle);
            let app_names: Vec<String> = apps
                .iter()
                .map(|a| wstring_to_string(&a.app_name))
                .filter(|s| !s.is_empty())
                .collect();
            
            profiles.push(NvidiaProfile {
                name,
                applications: app_names,
                settings: HashMap::new(),
                is_predefined,
            });
        }
        
        Ok(profiles)
    }

    /// Get a specific profile by name or executable
    pub fn get_profile(&self, name_or_exe: &str) -> Result<Option<NvidiaProfile>> {
        let profiles = self.list_profiles()?;

        Ok(profiles.into_iter().find(|p| {
            p.name.to_lowercase() == name_or_exe.to_lowercase()
                || p.applications.iter().any(|a| a.to_lowercase() == name_or_exe.to_lowercase())
        }))
    }

    /// Get a profile by name or executable, populating its current setting values
    /// by reading the known NVAPI DRS IDs for each field. Used for backup snapshots.
    pub fn get_profile_full(&self, name_or_exe: &str) -> Result<Option<NvidiaProfile>> {
        let session = match &self.session {
            Some(s) => s,
            None => return Ok(None),
        };

        let lookup_name = name_or_exe.to_lowercase();
        let mut target: Option<(NvDRSProfileHandle, String, bool, Vec<String>)> = None;

        for (handle, info) in session.enumerate_profiles() {
            let name = wstring_to_string(&info.profile_name);
            let apps = session.get_applications(handle);
            let app_names: Vec<String> = apps
                .iter()
                .map(|a| wstring_to_string(&a.app_name))
                .filter(|s| !s.is_empty())
                .collect();

            let matches = name.to_lowercase() == lookup_name
                || app_names.iter().any(|a| a.to_lowercase() == lookup_name);

            if matches {
                target = Some((handle, name, info.is_predefined != 0, app_names));
                break;
            }
        }

        let (handle, name, is_predefined, applications) = match target {
            Some(t) => t,
            None => return Ok(None),
        };

        let settings = read_known_settings(session, handle);
        Ok(Some(NvidiaProfile {
            name,
            applications,
            settings,
            is_predefined,
        }))
    }

    /// Create or update an application profile.
    ///
    /// Finds (or creates) a profile matching `profile.name`, attaches any missing
    /// executables, and writes every `Some(_)` field via `NvAPI_DRS_SetSetting`.
    /// Changes are flushed with a single `NvAPI_DRS_SaveSettings` at the end.
    pub fn set_profile(&self, profile: &DriverProfile) -> Result<()> {
        if profile.name.is_empty() && profile.executables.is_empty() {
            anyhow::bail!("Profile must have a name or at least one executable");
        }

        let session = self
            .session
            .as_ref()
            .context("DRS session unavailable - cannot write NVIDIA profile")?;

        // Find-or-create by name (fall back to first executable if unnamed).
        let profile_name = if profile.name.is_empty() {
            profile.executables[0].clone()
        } else {
            profile.name.clone()
        };

        let (handle, created) = match session.find_profile(&profile_name) {
            Some(h) => (h, false),
            None => {
                let h = session
                    .create_profile(&profile_name)
                    .with_context(|| format!("Failed to create NVIDIA profile '{}'", profile_name))?;
                (h, true)
            }
        };

        if created {
            tracing::info!("Created new NVIDIA profile '{}'", profile_name);
        }

        // Attach any executables that aren't already on the profile.
        let existing_apps: Vec<String> = session
            .get_applications(handle)
            .iter()
            .map(|a| wstring_to_string(&a.app_name).to_lowercase())
            .collect();

        for exe in &profile.executables {
            if existing_apps.contains(&exe.to_lowercase()) {
                continue;
            }
            if session.add_application(handle, exe) {
                tracing::info!("Attached executable '{}' to profile '{}'", exe, profile_name);
            } else {
                tracing::warn!("Failed to attach executable '{}' to profile '{}'", exe, profile_name);
            }
        }

        // Build the list of (id, value) writes based on Some(_) fields.
        let ops = driver_profile_writes(profile);
        if ops.is_empty() {
            tracing::debug!("No NVIDIA settings to write for profile '{}'", profile_name);
        }

        let mut applied = 0usize;
        for (setting_id, value, label) in &ops {
            let status = session.set_setting_dword_status(handle, *setting_id, *value);
            if status == 0 {
                tracing::info!(
                    "Set {} (0x{:08X}) = 0x{:08X} on '{}'",
                    label, setting_id, value, profile_name
                );
                applied += 1;
            } else {
                tracing::warn!(
                    "Failed to set {} (0x{:08X}) = 0x{:08X} on '{}' (NVAPI {} = {})",
                    label, setting_id, value, profile_name, status, status_message(status)
                );
            }
        }

        if !session.save() {
            anyhow::bail!("NvAPI_DRS_SaveSettings failed for profile '{}'", profile_name);
        }

        tracing::info!(
            "Saved NVIDIA profile '{}' ({} of {} settings applied)",
            profile_name, applied, ops.len()
        );

        Ok(())
    }

    /// Delete an entire application profile by name.
    ///
    /// Predefined (NVIDIA-shipped) profiles cannot be deleted; the driver will
    /// reject the call in that case and this method will return an error.
    pub fn delete_profile(&self, name: &str) -> Result<()> {
        let session = self
            .session
            .as_ref()
            .context("DRS session unavailable - cannot delete NVIDIA profile")?;

        let handle = session
            .find_profile(name)
            .with_context(|| format!("Profile '{}' not found", name))?;

        let status = session.delete_profile(handle);
        if status != 0 {
            anyhow::bail!(
                "NvAPI_DRS_DeleteProfile failed for '{}' (status={}). Predefined profiles cannot be deleted.",
                name, status
            );
        }

        if !session.save() {
            anyhow::bail!("NvAPI_DRS_SaveSettings failed after deleting '{}'", name);
        }

        tracing::info!("Deleted NVIDIA profile '{}'", name);
        Ok(())
    }

    /// Remove a single executable from a profile. The profile itself is kept.
    pub fn remove_application(&self, profile_name: &str, app_exe: &str) -> Result<()> {
        let session = self
            .session
            .as_ref()
            .context("DRS session unavailable")?;

        let handle = session
            .find_profile(profile_name)
            .with_context(|| format!("Profile '{}' not found", profile_name))?;

        let status = session.delete_application(handle, app_exe);
        if status != 0 {
            anyhow::bail!(
                "NvAPI_DRS_DeleteApplication failed for '{}' on '{}' (status={})",
                app_exe, profile_name, status
            );
        }

        if !session.save() {
            anyhow::bail!("NvAPI_DRS_SaveSettings failed after detaching '{}'", app_exe);
        }

        tracing::info!("Removed '{}' from profile '{}'", app_exe, profile_name);
        Ok(())
    }

    /// Get a single setting value from a named profile.
    pub fn get_setting(&self, profile_name: &str, setting: NvidiaSetting) -> Result<Option<u32>> {
        let session = match &self.session {
            Some(s) => s,
            None => return Ok(None),
        };

        let handle = match session.find_profile(profile_name) {
            Some(h) => h,
            None => return Ok(None),
        };

        // Safe: NVDRS_SETTING contains a union; interpret as DWORD.
        Ok(session
            .get_setting(handle, setting.setting_id())
            .map(|s| unsafe { s.current_value.u32_value }))
    }

    /// Set a single setting value in a profile. Creates the profile if needed.
    pub fn set_setting(&self, profile_name: &str, setting: NvidiaSetting, value: u32) -> Result<()> {
        let session = self
            .session
            .as_ref()
            .context("DRS session unavailable - cannot write NVIDIA setting")?;

        let setting_id = setting.setting_id();

        let handle = match session.find_profile(profile_name) {
            Some(h) => h,
            None => session
                .create_profile(profile_name)
                .with_context(|| format!("Failed to create NVIDIA profile '{}'", profile_name))?,
        };

        if !session.set_setting_dword(handle, setting_id, value) {
            anyhow::bail!(
                "NvAPI_DRS_SetSetting failed for {} on '{}'",
                setting.display_name(),
                profile_name
            );
        }
        if !session.save() {
            anyhow::bail!("NvAPI_DRS_SaveSettings failed for profile '{}'", profile_name);
        }
        Ok(())
    }

    /// Get the base/global profile
    pub fn get_base_profile(&self) -> Result<NvidiaProfile> {
        Ok(NvidiaProfile {
            name: "Base Profile".to_string(),
            applications: vec!["*".to_string()],
            settings: HashMap::new(),
            is_predefined: true,
        })
    }

    /// Reset a profile to default settings by writing each known setting's
    /// predefined (driver-default) value back.
    pub fn reset_profile(&self, name: &str) -> Result<()> {
        let session = self
            .session
            .as_ref()
            .context("DRS session unavailable - cannot reset NVIDIA profile")?;

        let handle = session
            .find_profile(name)
            .with_context(|| format!("Profile '{}' not found", name))?;

        let ids = [
            nvapi_ids::PRERENDERLIMIT,
            nvapi_ids::FRL_FPS,
            nvapi_ids::VSYNCMODE,
            nvapi_ids::VSYNC_TEAR_CONTROL,
            nvapi_ids::OGL_THREAD_CONTROL,
            nvapi_ids::OGL_TRIPLE_BUFFER,
            nvapi_ids::SHADER_DISK_CACHE,
            nvapi_ids::PREFERRED_PSTATE,
            nvapi_ids::ANISO_MODE_LEVEL,
            nvapi_ids::ANISO_MODE_SELECTOR,
            nvapi_ids::MAXWELL_B_SAMPLE_INTERLEAVE,
            nvapi_ids::QUALITY_ENHANCEMENTS,
            nvapi_ids::LODBIASADJUST,
            nvapi_ids::REFRESH_RATE_OVERRIDE,
            nvapi_ids::CUDA_FORCE_P2_STATE,
        ];

        let mut reset_count = 0usize;
        for id in ids {
            if let Some(setting) = session.get_setting(handle, id) {
                // predefined_value holds the NVIDIA-supplied default when valid.
                if setting.is_predefined_valid != 0 {
                    let default_val = unsafe { setting.predefined_value.u32_value };
                    if session.set_setting_dword(handle, id, default_val) {
                        reset_count += 1;
                    }
                }
            }
        }

        if !session.save() {
            anyhow::bail!("NvAPI_DRS_SaveSettings failed while resetting '{}'", name);
        }

        tracing::info!("Reset {} settings to defaults on profile '{}'", reset_count, name);
        Ok(())
    }
}

/// Build the list of `(setting_id, raw_value, label)` writes for a `DriverProfile`.
///
/// Only fields with a known NVAPI ID are emitted. Unknown mappings are logged and skipped.
fn driver_profile_writes(profile: &DriverProfile) -> Vec<(u32, u32, &'static str)> {
    let mut out: Vec<(u32, u32, &'static str)> = Vec::new();

    if let Some(fps) = profile.frame_rate_limit {
        out.push((nvapi_ids::FRL_FPS, fps, "Frame Rate Limit"));
    }
    if let Some(ll) = profile.low_latency {
        // 0 = off/app-controlled, 1 = on, 2 = ultra (maps to max pre-rendered frames)
        out.push((nvapi_ids::PRERENDERLIMIT, ll, "Low Latency Mode"));
    }
    if let Some(vsync) = profile.vsync {
        out.push((nvapi_ids::VSYNCMODE, vsync.to_nvapi_value(), "VSync Mode"));
        if let Some(tear) = vsync.tear_control_value() {
            out.push((nvapi_ids::VSYNC_TEAR_CONTROL, tear, "VSync Tear Control"));
        }
    }
    if let Some(pm) = profile.power_management {
        out.push((
            nvapi_ids::PREFERRED_PSTATE,
            pm.to_nvapi_value(),
            "Power Management",
        ));
    }
    if let Some(cache_on) = profile.shader_cache {
        let v = if cache_on { shader_cache_values::ON } else { shader_cache_values::OFF };
        out.push((nvapi_ids::SHADER_DISK_CACHE, v, "Shader Cache"));
    }
    if let Some(threaded) = profile.threaded_optimization {
        let v = if threaded {
            thread_control_values::ENABLE
        } else {
            thread_control_values::DISABLE
        };
        out.push((nvapi_ids::OGL_THREAD_CONTROL, v, "Threaded Optimization"));
    }
    if let Some(aniso) = profile.anisotropic_filtering {
        // Writing the selector ensures the level we set is actually applied
        // (otherwise app-controlled mode ignores our level value).
        out.push((
            nvapi_ids::ANISO_MODE_SELECTOR,
            aniso.to_selector_value(),
            "Anisotropic Filtering Mode",
        ));
        out.push((
            nvapi_ids::ANISO_MODE_LEVEL,
            aniso.to_nvapi_value(),
            "Anisotropic Filtering",
        ));
    }
    if let Some(mfaa) = profile.mfaa {
        let v = if mfaa { mfaa_values::ON } else { mfaa_values::OFF };
        out.push((nvapi_ids::MAXWELL_B_SAMPLE_INTERLEAVE, v, "MFAA"));
    }
    if let Some(triple) = profile.triple_buffering {
        let v = if triple {
            triple_buffer_values::ENABLED
        } else {
            triple_buffer_values::DISABLED
        };
        out.push((nvapi_ids::OGL_TRIPLE_BUFFER, v, "Triple Buffering"));
    }
    if let Some(quality) = profile.texture_filter_quality {
        out.push((
            nvapi_ids::QUALITY_ENHANCEMENTS,
            quality.to_nvapi_value(),
            "Texture Filtering - Quality",
        ));
    }
    if let Some(highest) = profile.preferred_refresh_rate {
        // 0 = Application Controlled, 1 = Highest Available.
        let v = if highest { 1 } else { 0 };
        out.push((nvapi_ids::REFRESH_RATE_OVERRIDE, v, "Preferred Refresh Rate"));
    }
    if let Some(fps) = profile.background_fps_limit {
        out.push((
            nvapi_ids::APPIDLE_DYNAMIC_FRL_FPS,
            fps,
            "Background Max FPS",
        ));
    }

    out
}

/// Read the subset of known DRS settings from a profile handle into a `HashMap`
/// keyed by the human-readable display name.
fn read_known_settings(
    session: &DrsSession,
    handle: NvDRSProfileHandle,
) -> HashMap<String, ProfileSettingValue> {
    let ids: &[(u32, NvidiaSetting)] = &[
        (nvapi_ids::PRERENDERLIMIT, NvidiaSetting::PreRenderLimit),
        (nvapi_ids::FRL_FPS, NvidiaSetting::FrameRateLimit),
        (nvapi_ids::VSYNCMODE, NvidiaSetting::VSync),
        (nvapi_ids::OGL_THREAD_CONTROL, NvidiaSetting::ThreadedOptimization),
        (nvapi_ids::OGL_TRIPLE_BUFFER, NvidiaSetting::TripleBuffering),
        (nvapi_ids::SHADER_DISK_CACHE, NvidiaSetting::ShaderCache),
        (nvapi_ids::PREFERRED_PSTATE, NvidiaSetting::PowerManagement),
        (nvapi_ids::ANISO_MODE_LEVEL, NvidiaSetting::AnisotropicFiltering),
        (nvapi_ids::MAXWELL_B_SAMPLE_INTERLEAVE, NvidiaSetting::Mfaa),
        (nvapi_ids::QUALITY_ENHANCEMENTS, NvidiaSetting::TextureFilteringQuality),
        (nvapi_ids::LODBIASADJUST, NvidiaSetting::TextureFilteringLodBias),
        (nvapi_ids::REFRESH_RATE_OVERRIDE, NvidiaSetting::PreferredRefreshRate),
        (nvapi_ids::CUDA_FORCE_P2_STATE, NvidiaSetting::CudaForceP2State),
    ];

    let mut out = HashMap::new();
    for (id, setting) in ids {
        if let Some(s) = session.get_setting(handle, *id) {
            let value = unsafe { s.current_value.u32_value };
            out.insert(
                setting.display_name().to_string(),
                ProfileSettingValue {
                    id: format!("0x{:08X}", id),
                    value,
                    display_value: Some(describe_setting_value(*setting, value)),
                },
            );
        }
    }
    out
}

/// Produce a human-readable rendering for a known setting's raw DWORD value.
fn describe_setting_value(setting: NvidiaSetting, value: u32) -> String {
    match setting {
        NvidiaSetting::VSync => match VSyncMode::from_nvapi_value(value) {
            Some(v) => format!("{:?}", v),
            None => format!("0x{:08X}", value),
        },
        NvidiaSetting::PowerManagement => match PowerManagementMode::from_nvapi_value(value) {
            Some(v) => format!("{:?}", v),
            None => format!("0x{:08X}", value),
        },
        NvidiaSetting::ShaderCache => match value {
            shader_cache_values::OFF => "Off".into(),
            shader_cache_values::ON => "On".into(),
            other => format!("0x{:08X}", other),
        },
        NvidiaSetting::ThreadedOptimization => match value {
            thread_control_values::DEFAULT => "Auto".into(),
            thread_control_values::ENABLE => "On".into(),
            thread_control_values::DISABLE => "Off".into(),
            other => format!("0x{:08X}", other),
        },
        NvidiaSetting::TripleBuffering => match value {
            triple_buffer_values::DISABLED => "Off".into(),
            triple_buffer_values::ENABLED => "On".into(),
            other => format!("0x{:08X}", other),
        },
        NvidiaSetting::Mfaa => match value {
            mfaa_values::OFF => "Off".into(),
            mfaa_values::ON => "On".into(),
            other => format!("0x{:08X}", other),
        },
        NvidiaSetting::AnisotropicFiltering => match AnisotropicLevel::from_nvapi_value(value) {
            Some(AnisotropicLevel::ApplicationControlled) => "App Controlled".into(),
            Some(level) => format!("{}x", level.to_value()),
            None => format!("0x{:08X}", value),
        },
        NvidiaSetting::TextureFilteringQuality => {
            match TextureFilterQuality::from_nvapi_value(value) {
                Some(TextureFilterQuality::HighQuality) => "High Quality".into(),
                Some(TextureFilterQuality::Quality) => "Quality".into(),
                Some(TextureFilterQuality::Performance) => "Performance".into(),
                Some(TextureFilterQuality::HighPerformance) => "High Performance".into(),
                None => format!("0x{:08X}", value),
            }
        }
        NvidiaSetting::PreferredRefreshRate => match value {
            0 => "Application Controlled".into(),
            1 => "Highest Available".into(),
            other => format!("{}", other),
        },
        NvidiaSetting::CudaForceP2State => {
            if value == 0 { "Off".into() } else { "On".into() }
        }
        NvidiaSetting::FrameRateLimit => {
            if value == 0 {
                "Off".into()
            } else {
                format!("{} FPS", value)
            }
        }
        NvidiaSetting::PreRenderLimit => match value {
            0 => "Off (app controlled)".into(),
            1 => "On".into(),
            2 => "Ultra".into(),
            other => format!("{} frames", other),
        },
        _ => value.to_string(),
    }
}

impl Default for NvidiaProfileManager {
    fn default() -> Self {
        Self::new().expect("Failed to initialize NVIDIA profile manager")
    }
}

/// Check if NVIDIA driver is installed and accessible
pub fn is_nvidia_available() -> bool {
    nvapi::initialize().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_nvidia_available() {
        // This test will only pass on systems with NVIDIA GPUs
        let available = is_nvidia_available();
        println!("NVIDIA available: {}", available);
    }
}
