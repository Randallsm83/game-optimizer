//! Data sources for game information

pub mod pcgamingwiki;
pub mod steam;
pub mod xbox;
pub mod gog;
pub mod epic;
pub mod ea;
pub mod games;
pub mod cache;
pub mod benchmarks;

pub use games::{detect_all_games, find_games_by_name, DetectedGame, GamePlatform};
pub use benchmarks::{BenchmarkClient, BenchmarkArticle, BenchmarkSource, RecommendedSettings};
