//! AI-powered game optimization recommendations
//!
//! Uses Claude API to analyze hardware and game data to generate
//! optimal settings recommendations.

pub mod claude;
pub mod prompts;

pub use claude::{ClaudeClient, ClaudeError};
pub use prompts::{RecommendationRequest, RecommendationResponse, DriverSettings, InGameSettings};
