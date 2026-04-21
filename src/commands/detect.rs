//! GPU detection commands

use anyhow::Result;

use crate::hardware::gpu::{enumerate_gpus, GpuInfo};

/// Detect and display primary GPU information
pub fn detect_gpu(json_output: bool) -> Result<()> {
    let gpu = GpuInfo::detect()?;

    if json_output {
        println!("{}", serde_json::to_string_pretty(&gpu)?);
    } else {
        println!("🎮 GPU Detected");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("  Name:         {}", gpu.name);
        if let Some(arch) = &gpu.architecture {
            println!("  Architecture: {}", arch);
        }
        println!("  VRAM:         {}", gpu.vram_display);
        println!("  Driver:       {}", gpu.driver_version);
        if let Some(bios) = &gpu.bios_version {
            println!("  BIOS:         {}", bios);
        }
        if let Some(temp) = gpu.temperature_c {
            println!("  Temperature:  {}°C", temp);
        }
        if let Some(core_clock) = gpu.core_clock_mhz {
            println!("  Core Clock:   {} MHz", core_clock);
        }
        if let Some(mem_clock) = gpu.memory_clock_mhz {
            println!("  Memory Clock: {} MHz", mem_clock);
        }
    }

    Ok(())
}

/// List all detected NVIDIA GPUs
pub fn list_gpus(json_output: bool) -> Result<()> {
    let gpus = enumerate_gpus()?;

    if json_output {
        println!("{}", serde_json::to_string_pretty(&gpus)?);
    } else {
        println!("🎮 Detected GPUs");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        if gpus.is_empty() {
            println!("  No NVIDIA GPUs found");
        } else {
            for (i, gpu) in gpus.iter().enumerate() {
                println!("  [{}] {}", i, gpu.summary());
            }
        }
    }

    Ok(())
}
