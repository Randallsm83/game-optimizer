//! External tool integrations
//!
//! Integration with framerate limiting and frame generation tools:
//! - RTSS (RivaTuner Statistics Server) for framerate limiting
//! - Lossless Scaling for frame generation (LSFG)

pub mod lossless;
pub mod rtss;

pub use lossless::{LosslessScaling, LsProfile, LsSettings, LsfgMode, LsfgMultiplier, ScalingType};
pub use rtss::{RtssManager, RtssProfile};
