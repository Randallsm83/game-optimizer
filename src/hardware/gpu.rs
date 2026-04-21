//! GPU detection and information via NVAPI

use anyhow::{Context, Result};
use nvapi;
use serde::{Deserialize, Serialize};

/// GPU information retrieved via NVAPI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    /// GPU full name (e.g., "NVIDIA GeForce RTX 5080")
    pub name: String,

    /// GPU short name/codename
    pub codename: Option<String>,

    /// Total VRAM in bytes
    pub vram_bytes: u64,

    /// VRAM in human-readable format
    pub vram_display: String,

    /// Driver version string
    pub driver_version: String,

    /// BIOS version
    pub bios_version: Option<String>,

    /// GPU core count
    pub cuda_cores: Option<u32>,

    /// Current GPU clock speed (MHz)
    pub core_clock_mhz: Option<u32>,

    /// Current memory clock speed (MHz)
    pub memory_clock_mhz: Option<u32>,

    /// GPU temperature (Celsius)
    pub temperature_c: Option<i32>,

    /// GPU architecture
    pub architecture: Option<String>,
}

impl GpuInfo {
    /// Detect the primary NVIDIA GPU using NVAPI
    pub fn detect() -> Result<Self> {
        // Initialize NVAPI
        nvapi::initialize().context("Failed to initialize NVAPI")?;

        // Get all physical GPUs
        let gpus = nvapi::PhysicalGpu::enumerate()
            .context("Failed to enumerate GPUs")?;

        let gpu = gpus.first()
            .context("No NVIDIA GPU found")?;

        // Get GPU name
        let name = gpu.full_name()
            .context("Failed to get GPU name")?;

        // Get VRAM info - use dedicated field from MemoryInfo
        let memory_info = gpu.memory_info()
            .context("Failed to get memory info")?;
        let vram_bytes = (memory_info.dedicated.0 as u64) * 1024; // Kibibytes to bytes
        let vram_display = format_bytes(vram_bytes);

        // Get driver version using correct function name
        let driver_version = nvapi::driver_version()
            .map(|(ver, _branch)| {
                // ver is like 59174 which means 591.74
                let v = format!("{}", ver);
                if v.len() >= 3 {
                    format!("{}.{}", &v[..v.len()-2], &v[v.len()-2..])
                } else {
                    v
                }
            })
            .unwrap_or_else(|_| "Unknown".to_string());

        // Try to get additional info (may not be available on all GPUs)
        let bios_version = gpu.vbios_version_string().ok();

        // Get thermal info if available
        let temperature_c = gpu.thermal_settings(None)
            .ok()
            .and_then(|temps| temps.first().map(|t| t.current_temperature.0));

        // Get clock speeds if available - use ClockDomain enum
        let clocks = gpu.clock_frequencies(nvapi::ClockFrequencyType::Current).ok();
        let core_clock_mhz = clocks.as_ref()
            .and_then(|c| c.get(&nvapi::ClockDomain::Graphics).map(|f| f.0 / 1000));
        let memory_clock_mhz = clocks.as_ref()
            .and_then(|c| c.get(&nvapi::ClockDomain::Memory).map(|f| f.0 / 1000));

        let architecture = detect_architecture(&name);
        Ok(GpuInfo {
            name,
            codename: None,
            vram_bytes,
            vram_display,
            driver_version,
            bios_version,
            cuda_cores: None,
            core_clock_mhz,
            memory_clock_mhz,
            temperature_c,
            architecture,
        })
    }

    /// Get a summary string for display
    pub fn summary(&self) -> String {
        let mut parts = vec![self.name.clone()];
        parts.push(format!("VRAM: {}", self.vram_display));
        parts.push(format!("Driver: {}", self.driver_version));
        if let Some(temp) = self.temperature_c {
            parts.push(format!("Temp: {}°C", temp));
        }
        parts.join(" | ")
    }
}

/// Format bytes as human-readable string
fn format_bytes(bytes: u64) -> String {
    const GB: u64 = 1024 * 1024 * 1024;
    const MB: u64 = 1024 * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    }
}

/// Try to detect GPU architecture from name
fn detect_architecture(name: &str) -> Option<String> {
    let name_lower = name.to_lowercase();
    
    if name_lower.contains("rtx 50") {
        Some("Blackwell".to_string())
    } else if name_lower.contains("rtx 40") {
        Some("Ada Lovelace".to_string())
    } else if name_lower.contains("rtx 30") {
        Some("Ampere".to_string())
    } else if name_lower.contains("rtx 20") || name_lower.contains("gtx 16") {
        Some("Turing".to_string())
    } else if name_lower.contains("gtx 10") {
        Some("Pascal".to_string())
    } else {
        None
    }
}

/// Get all available NVIDIA GPUs
pub fn enumerate_gpus() -> Result<Vec<GpuInfo>> {
    nvapi::initialize().context("Failed to initialize NVAPI")?;
    
    let gpus = nvapi::PhysicalGpu::enumerate()
        .context("Failed to enumerate GPUs")?;

    let mut results = Vec::new();
    for gpu in gpus {
        if let Ok(info) = gpu_info_from_physical(&gpu) {
            results.push(info);
        }
    }

    Ok(results)
}

fn gpu_info_from_physical(gpu: &nvapi::PhysicalGpu) -> Result<GpuInfo> {
    let name = gpu.full_name().context("Failed to get GPU name")?;
    let memory_info = gpu.memory_info().context("Failed to get memory info")?;
    let vram_bytes = (memory_info.dedicated.0 as u64) * 1024; // Kibibytes to bytes

    let driver_version = nvapi::driver_version()
        .map(|(ver, _branch)| {
            let v = format!("{}", ver);
            if v.len() >= 3 {
                format!("{}.{}", &v[..v.len()-2], &v[v.len()-2..])
            } else {
                v
            }
        })
        .unwrap_or_else(|_| "Unknown".to_string());

    Ok(GpuInfo {
        name: name.clone(),
        codename: None,
        vram_bytes,
        vram_display: format_bytes(vram_bytes),
        driver_version,
        bios_version: gpu.vbios_version_string().ok(),
        cuda_cores: None,
        core_clock_mhz: None,
        memory_clock_mhz: None,
        temperature_c: gpu.thermal_settings(None)
            .ok()
            .and_then(|temps| temps.first().map(|t| t.current_temperature.0)),
        architecture: detect_architecture(&name),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(16 * 1024 * 1024 * 1024), "16.0 GB");
        assert_eq!(format_bytes(8 * 1024 * 1024 * 1024), "8.0 GB");
        assert_eq!(format_bytes(512 * 1024 * 1024), "512.0 MB");
    }

    #[test]
    fn test_detect_architecture() {
        assert_eq!(detect_architecture("GeForce RTX 5080"), Some("Blackwell".to_string()));
        assert_eq!(detect_architecture("GeForce RTX 4090"), Some("Ada Lovelace".to_string()));
        assert_eq!(detect_architecture("GeForce RTX 3080"), Some("Ampere".to_string()));
    }
}
