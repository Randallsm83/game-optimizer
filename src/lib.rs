//! Game Optimizer - AI-powered game settings optimizer
//!
//! This library provides functionality for:
//! - Detecting GPU hardware via NVAPI
//! - Managing NVIDIA driver profiles
//! - Integrating with RTSS for framerate limiting
//! - Managing Lossless Scaling profiles
//! - AI-powered settings recommendations

pub mod config;
pub mod hardware;
pub mod drivers;
pub mod data;
pub mod commands;
pub mod ai;
pub mod tools;
pub mod backup;

// Re-exports for convenience
pub use config::Config;
pub use hardware::gpu::GpuInfo;
pub use ai::{ClaudeClient, RecommendationRequest, RecommendationResponse};
pub use tools::{LosslessScaling, RtssManager};
