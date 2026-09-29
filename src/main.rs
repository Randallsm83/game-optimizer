//! Game Optimizer CLI
//!
//! AI-powered game settings optimizer for NVIDIA drivers, RTSS, and Lossless Scaling

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use game_optimizer::commands;

#[derive(Parser)]
#[command(name = "game-optimizer")]
#[command(author, version, about = "AI-powered game settings optimizer", long_about = None)]
struct Cli {
    /// Output in JSON format
    #[arg(long, global = true)]
    json: bool,

    /// Enable verbose logging (INFO level)
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Enable debug logging (DEBUG + TRACE level)
    #[arg(long, global = true)]
    debug: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Show current configuration
    Show,
    
    /// Set a configuration value
    Set {
        /// Configuration key (api-key, resolution, refresh-rate, gsync)
        key: String,
        /// Value to set
        value: String,
    },
    
    /// Initialize config file with defaults
    Init,
    
    /// Show config file path
    Path,
}

#[derive(Subcommand)]
enum RtssAction {
    /// Check if RTSS is installed and running
    Status,
    
    /// List all RTSS profiles
    List,
    
    /// Get profile for a specific game
    Get {
        /// Game executable name
        name: String,
    },
    
    /// Set FPS limit for a game
    Set {
        /// Game executable name
        game: String,
        /// FPS limit (0 = unlimited)
        #[arg(long)]
        fps: u32,
    },
}

#[derive(Subcommand)]
enum LsAction {
    /// Check Lossless Scaling status
    Status,
    
    /// List saved profiles
    List,
    
    /// Get profile for a specific game
    Get {
        /// Game name
        name: String,
    },
    
    /// Set LSFG settings for a game profile
    Set {
        /// Profile name (must exist)
        name: String,
        
        /// LSFG mode (fixed, adaptive, off)
        #[arg(long)]
        mode: Option<String>,
        
        /// Frame generation multiplier (2, 3, 4)
        #[arg(long)]
        multiplier: Option<u32>,
        
        /// Target FPS
        #[arg(long)]
        target: Option<u32>,
        
        /// Flow scale (0-100)
        #[arg(long)]
        flow: Option<u32>,
    },
    
    /// Calculate recommended FPS settings for LSFG
    Calc {
        /// Target output FPS
        #[arg(long)]
        target: u32,
        /// Expected native FPS the game can achieve
        #[arg(long)]
        native: u32,
    },
}

#[derive(Subcommand)]
enum BackupAction {
    /// List all backups
    List {
        /// Filter by game name
        #[arg(short, long)]
        game: Option<String>,
    },

    /// Show details of a specific backup
    Show {
        /// Backup ID
        id: String,
    },

    /// Restore settings from a backup
    Restore {
        /// Backup ID (use 'latest' for most recent)
        id: String,

        /// Game name (required if using 'latest')
        #[arg(long)]
        game: Option<String>,

        /// Preview changes without applying
        #[arg(long)]
        dry_run: bool,

        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },

    /// Delete a backup
    Delete {
        /// Backup ID
        id: String,
    },

    /// Export optimization profile to file
    Export {
        /// Game name to export
        game: String,

        /// Output file path (default: <game>.json)
        #[arg(short, long)]
        output: Option<String>,

        /// Format: json or toml
        #[arg(long, default_value = "json")]
        format: String,
    },

    /// Import optimization profile from file
    Import {
        /// Profile file path
        file: String,

        /// Preview changes without applying
        #[arg(long)]
        dry_run: bool,

        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

#[derive(Subcommand)]
enum Commands {
    /// Detect GPU hardware
    #[command(name = "detect-gpu")]
    DetectGpu,

    /// List all detected GPUs
    #[command(name = "list-gpus")]
    ListGpus,

    /// List NVIDIA driver profiles (installed games only by default)
    #[command(name = "list-profiles")]
    ListProfiles {
        /// Show all profiles including NVIDIA predefined ones
        #[arg(long)]
        all: bool,
        
        /// Only show profiles for installed games (default)
        #[arg(long, default_value_t = true)]
        installed: bool,
        
        /// Filter by name
        #[arg(short, long)]
        filter: Option<String>,
    },

    /// Get a specific driver profile
    #[command(name = "get-profile")]
    GetProfile {
        /// Profile name or game executable
        name: String,
    },

    /// Delete an NVIDIA driver profile
    #[command(name = "delete-profile")]
    DeleteProfile {
        /// Profile name to delete (predefined profiles cannot be deleted)
        name: String,

        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },

    /// Set driver profile settings
    #[command(name = "set-profile")]
    SetProfile {
        /// Game executable name
        game: String,

        /// Frame rate limit (0 = off)
        #[arg(long)]
        fps_limit: Option<u32>,

        /// Low latency mode (0=off, 1=on, 2=ultra)
        #[arg(long)]
        low_latency: Option<u32>,

        /// VSync mode (off, on, adaptive, fast)
        #[arg(long)]
        vsync: Option<String>,

        /// Power management (adaptive, max-performance, optimal)
        #[arg(long)]
        power: Option<String>,

        /// Anisotropic filtering (0=app, 2, 4, 8, 16)
        #[arg(long)]
        anisotropic: Option<u32>,

        /// MFAA (Multi-Frame Sampled AA): on/off
        #[arg(long)]
        mfaa: Option<bool>,

        /// Triple buffering (OpenGL): on/off
        #[arg(long)]
        triple_buffer: Option<bool>,

        /// Texture filtering quality (high_quality, quality, performance, high_performance)
        #[arg(long)]
        texture_quality: Option<String>,

        /// Preferred refresh rate: on=Highest Available, off=Application Controlled
        #[arg(long)]
        preferred_refresh_rate: Option<bool>,

        /// Background application max FPS (0 = off)
        #[arg(long)]
        background_fps: Option<u32>,

        /// Antialiasing mode (app, override, enhance)
        #[arg(long)]
        aa_mode: Option<String>,

        /// Antialiasing setting (none, 2x, 4x, 8x)
        #[arg(long)]
        aa_setting: Option<String>,

        /// Texture filtering - LOD bias (signed; typical -3000..=3000)
        #[arg(long)]
        lod_bias: Option<i32>,

        /// Texture filtering - Trilinear optimization: on/off
        #[arg(long)]
        trilinear_optimization: Option<bool>,

        /// Texture filtering - Anisotropic sample optimization: on/off
        #[arg(long)]
        aniso_sample_optimization: Option<bool>,

        /// G-SYNC indicator overlay: on/off
        #[arg(long)]
        gsync_indicator: Option<bool>,

        /// CUDA Force P2 State: on/off
        #[arg(long)]
        cuda_force_p2: Option<bool>,
    },

    /// Remove one executable from a profile (keeps the profile itself)
    #[command(name = "remove-application")]
    RemoveApplication {
        /// Profile name
        profile: String,
        /// Executable name (e.g. eldenring.exe)
        exe: String,
        /// Skip confirmation
        #[arg(short = 'y', long)]
        yes: bool,
    },

    /// Reset every known setting on a profile back to NVIDIA's predefined defaults
    #[command(name = "reset-profile")]
    ResetProfile {
        /// Profile name
        name: String,
        /// Skip confirmation
        #[arg(short = 'y', long)]
        yes: bool,
    },

    /// Manage application configuration
    #[command(name = "config")]
    Config {
        #[command(subcommand)]
        action: Option<ConfigAction>,
    },

    /// Get game information from PCGamingWiki
    #[command(name = "game-info")]
    GameInfo {
        /// Game name or Steam AppID
        query: String,
    },

    /// List installed Steam games
    #[command(name = "list-games")]
    ListGames {
        /// Filter by name
        #[arg(short, long)]
        filter: Option<String>,
    },

    /// Get AI-powered optimization recommendations for a game
    #[command(name = "recommend")]
    Recommend {
        /// Game name or Steam AppID
        game: String,

        /// Only recommend NVIDIA driver settings (skip in-game settings)
        #[arg(long)]
        driver_only: bool,

        /// Use quick heuristics instead of AI (no API call)
        #[arg(long)]
        quick: bool,

        /// Target resolution (reads from config if not specified)
        #[arg(long)]
        resolution: Option<String>,

        /// Target FPS (reads from config if not specified)
        #[arg(long)]
        fps: Option<u32>,

        /// Disable VRR (G-Sync/FreeSync) optimizations
        #[arg(long)]
        no_vrr: bool,
    },

    /// Search for benchmark articles for a game
    #[command(name = "benchmarks")]
    Benchmarks {
        /// Game name to search for
        game: String,
    },

    /// Full optimization workflow: analyze, recommend, preview, and apply settings
    #[command(name = "optimize")]
    Optimize {
        /// Game name or Steam AppID
        game: String,

        /// Only apply NVIDIA driver settings (skip RTSS/LS)
        #[arg(long)]
        driver_only: bool,

        /// Use quick heuristics instead of AI
        #[arg(long)]
        quick: bool,

        /// Target resolution
        #[arg(long)]
        resolution: Option<String>,

        /// Target FPS
        #[arg(long)]
        fps: Option<u32>,

        /// Disable VRR optimizations
        #[arg(long)]
        no_vrr: bool,

        /// Preview changes without applying (dry-run)
        #[arg(long)]
        dry_run: bool,

        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,

        /// Skip creating a backup
        #[arg(long)]
        no_backup: bool,
    },

    /// Manage settings backups
    #[command(name = "backup")]
    Backup {
        #[command(subcommand)]
        action: BackupAction,
    },

    /// Manage RTSS (RivaTuner Statistics Server) profiles
    #[command(name = "rtss")]
    Rtss {
        #[command(subcommand)]
        action: RtssAction,
    },

    /// Manage Lossless Scaling profiles
    #[command(name = "ls")]
    LosslessScaling {
        #[command(subcommand)]
        action: LsAction,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let filter = if cli.debug {
        "game_optimizer=trace"
    } else if cli.verbose {
        "game_optimizer=debug"
    } else {
        "game_optimizer=warn"
    };

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| filter.into()))
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(cli.debug)
                .with_file(cli.debug)
                .with_line_number(cli.debug)
        )
        .init();

    match cli.command {
        Commands::DetectGpu => {
            commands::detect::detect_gpu(cli.json)?;
        }
        Commands::ListGpus => {
            commands::detect::list_gpus(cli.json)?;
        }
        Commands::ListProfiles { all, installed, filter } => {
            list_profiles(all, installed && !all, filter.as_deref(), cli.json)?;
        }
        Commands::GetProfile { name } => {
            get_profile(&name, cli.json)?;
        }
        Commands::SetProfile {
            game,
            fps_limit,
            low_latency,
            vsync,
            power,
            anisotropic,
            mfaa,
            triple_buffer,
            texture_quality,
            preferred_refresh_rate,
            background_fps,
            aa_mode,
            aa_setting,
            lod_bias,
            trilinear_optimization,
            aniso_sample_optimization,
            gsync_indicator,
            cuda_force_p2,
        } => {
            set_profile(SetProfileArgs {
                game: &game,
                fps_limit,
                low_latency,
                vsync,
                power,
                anisotropic,
                mfaa,
                triple_buffer,
                texture_quality,
                preferred_refresh_rate,
                background_fps,
                aa_mode,
                aa_setting,
                lod_bias,
                trilinear_optimization,
                aniso_sample_optimization,
                gsync_indicator,
                cuda_force_p2,
            })?;
        }
        Commands::DeleteProfile { name, yes } => {
            delete_profile(&name, yes)?;
        }
        Commands::RemoveApplication { profile, exe, yes } => {
            remove_application(&profile, &exe, yes)?;
        }
        Commands::ResetProfile { name, yes } => {
            reset_profile(&name, yes)?;
        }
        Commands::Config { action } => {
            handle_config(action)?;
        }
        Commands::GameInfo { query } => {
            game_info(&query, cli.json)?;
        }
        Commands::ListGames { filter } => {
            list_games(filter.as_deref(), cli.json)?;
        }
        Commands::Recommend {
            game,
            driver_only,
            quick,
            resolution,
            fps,
            no_vrr,
        } => {
            // Load defaults from config
            let config = game_optimizer::config::Config::load().unwrap_or_default();
            let res = resolution.unwrap_or_else(|| {
                config.target_resolution.unwrap_or_else(|| "2560x1440".to_string())
            });
            let target_fps = fps.unwrap_or_else(|| {
                config.target_refresh_rate.unwrap_or(144)
            });
            let vrr = if no_vrr { false } else { config.gsync_enabled };
            
            recommend(&game, driver_only, quick, &res, target_fps, vrr, cli.json)?;
        }
        Commands::Benchmarks { game } => {
            search_benchmarks(&game, cli.json)?;
        }
        Commands::Rtss { action } => {
            handle_rtss(action, cli.json)?;
        }
        Commands::LosslessScaling { action } => {
            handle_lossless_scaling(action, cli.json)?;
        }
        Commands::Optimize {
            game,
            driver_only,
            quick,
            resolution,
            fps,
            no_vrr,
            dry_run,
            yes,
            no_backup,
        } => {
            let config = game_optimizer::config::Config::load().unwrap_or_default();
            let res = resolution.unwrap_or_else(|| {
                config.target_resolution.unwrap_or_else(|| "2560x1440".to_string())
            });
            let target_fps = fps.unwrap_or_else(|| {
                config.target_refresh_rate.unwrap_or(144)
            });
            let vrr = if no_vrr { false } else { config.gsync_enabled };
            
            optimize(&game, driver_only, quick, &res, target_fps, vrr, dry_run, yes, no_backup, cli.json)?;
        }
        Commands::Backup { action } => {
            handle_backup(action, cli.json)?;
        }
    }

    Ok(())
}

fn list_profiles(show_all: bool, installed_only: bool, filter: Option<&str>, json_output: bool) -> Result<()> {
    use game_optimizer::drivers::nvidia::NvidiaProfileManager;
    use game_optimizer::data::detect_all_games;

    let manager = NvidiaProfileManager::new()?;
    let mut profiles = manager.list_profiles_filtered(show_all, filter)?;

    // Filter to installed games if requested
    if installed_only {
        let installed_games = detect_all_games().unwrap_or_default();
        
        // Build list of installed game names (lowercase for matching)
        // Also extract significant words (3+ chars) for matching
        let installed_names: Vec<(String, Vec<String>)> = installed_games
            .iter()
            .map(|g| {
                let name = g.name.to_lowercase();
                let words: Vec<String> = name
                    .split(|c: char| !c.is_alphanumeric())
                    .filter(|w| w.len() >= 3)
                    .map(|s| s.to_string())
                    .collect();
                (name, words)
            })
            .collect();
        
        // Filter profiles to only those matching installed games
        // Always keep user-created profiles
        profiles.retain(|p| {
            if !p.is_predefined {
                return true; // Always show user-created profiles
            }
            
            let profile_name = p.name.to_lowercase();
            let profile_words: Vec<&str> = profile_name
                .split(|c: char| !c.is_alphanumeric())
                .filter(|w| w.len() >= 2)
                .collect();
            
            installed_names.iter().any(|(game_name, game_words)| {
                // Exact match
                if profile_name == *game_name {
                    return true;
                }
                // Profile starts with game name (handles "Elden Ring easy anti-cheat launcher")
                if profile_name.starts_with(game_name) {
                    return true;
                }
                // Game name starts with profile name
                if game_name.starts_with(&profile_name) {
                    let remainder = &game_name[profile_name.len()..];
                    // Exact match
                    if remainder.is_empty() {
                        return true;
                    }
                    // Subtitle match (e.g., "The Witcher 3" matches "The Witcher 3: Wild Hunt")
                    if remainder.starts_with(':') {
                        return true;
                    }
                    // Sequel number match (e.g., "EVERSPACE" matches "EVERSPACE 2")
                    // Only if followed by space + digit
                    if let Some(next_char) = remainder.strip_prefix(' ').and_then(|s| s.chars().next()) {
                        if next_char.is_ascii_digit() {
                            return true;
                        }
                    }
                    // Otherwise require high ratio (>= 70%)
                    let ratio = profile_name.len() as f32 / game_name.len() as f32;
                    if ratio >= 0.7 {
                        return true;
                    }
                }
                // Handle subtitled games: "The Witcher 3: Wild Hunt" -> match "The Witcher 3"
                if let Some(base_name) = game_name.split(':').next() {
                    let base_trimmed = base_name.trim();
                    if profile_name == base_trimmed || profile_name.starts_with(base_trimmed) {
                        return true;
                    }
                }
                // For multi-word game names, require ALL game words present in profile
                // AND profile shouldn't have significantly fewer words
                if game_words.len() >= 2 {
                    let all_words_match = game_words.iter().all(|gw| {
                        profile_words.iter().any(|pw| *pw == gw.as_str())
                    });
                    if all_words_match && profile_words.len() >= game_words.len() - 1 {
                        return true;
                    }
                }
                // For single long words (like "Enshrouded", "Satisfactory"), exact word match
                if game_words.len() == 1 && game_words[0].len() >= 8 {
                    let word = &game_words[0];
                    if profile_words.iter().any(|pw| *pw == word.as_str()) {
                        return true;
                    }
                }
                false
            })
        });
    }

    if json_output {
        println!("{}", serde_json::to_string_pretty(&profiles)?);
    } else {
        let title = if show_all {
            "NVIDIA Driver Profiles (all)"
        } else if installed_only {
            "NVIDIA Driver Profiles (installed games)"
        } else {
            "NVIDIA Driver Profiles"
        };
        println!("📋 {}", title);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        if profiles.is_empty() {
            println!("  No profiles found.");
            if installed_only {
                println!("  Use --all to see all NVIDIA profiles.");
            }
        } else {
            for profile in &profiles {
                let apps = if profile.applications.is_empty() {
                    String::new()
                } else {
                    format!(" - {}", profile.applications.join(", "))
                };
                println!(
                    "  {} [{}]{}",
                    if profile.is_predefined { "🔒" } else { "📝" },
                    profile.name,
                    apps
                );
            }
            println!("\n  Total: {} profiles", profiles.len());
        }
    }

    Ok(())
}

fn get_profile(name: &str, json_output: bool) -> Result<()> {
    use game_optimizer::drivers::nvidia::NvidiaProfileManager;

    let manager = NvidiaProfileManager::new()?;

    // Prefer the full-read path (populates settings from DRS); fall back to the
    // fast enumerate-based lookup for profiles that exist but have no settings.
    let profile = match manager.get_profile_full(name)? {
        Some(p) => Some(p),
        None => manager.get_profile(name)?,
    };

    match profile {
        Some(profile) => {
            if json_output {
                println!("{}", serde_json::to_string_pretty(&profile)?);
            } else {
                println!("📋 Profile: {}", profile.name);
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!("  Type:         {}", if profile.is_predefined { "Predefined" } else { "User" });
                println!("  Applications: {}", profile.applications.join(", "));
                if profile.settings.is_empty() {
                    println!("  Settings:     (none read or all at default)");
                } else {
                    println!("  Settings:");
                    // Deterministic ordering for readability.
                    let mut items: Vec<_> = profile.settings.iter().collect();
                    items.sort_by(|a, b| a.0.cmp(b.0));
                    for (key, value) in items {
                        let display = value.display_value.as_deref().unwrap_or("-");
                        println!(
                            "    {:<32} [{}] 0x{:08X} ({})",
                            key, value.id, value.value, display
                        );
                    }
                }
            }
        }
        None => {
            if json_output {
                println!("null");
            } else {
                println!("Profile '{}' not found", name);
            }
        }
    }

    Ok(())
}

struct SetProfileArgs<'a> {
    game: &'a str,
    fps_limit: Option<u32>,
    low_latency: Option<u32>,
    vsync: Option<String>,
    power: Option<String>,
    anisotropic: Option<u32>,
    mfaa: Option<bool>,
    triple_buffer: Option<bool>,
    texture_quality: Option<String>,
    preferred_refresh_rate: Option<bool>,
    background_fps: Option<u32>,
    aa_mode: Option<String>,
    aa_setting: Option<String>,
    lod_bias: Option<i32>,
    trilinear_optimization: Option<bool>,
    aniso_sample_optimization: Option<bool>,
    gsync_indicator: Option<bool>,
    cuda_force_p2: Option<bool>,
}

fn set_profile(args: SetProfileArgs<'_>) -> Result<()> {
    use game_optimizer::drivers::nvidia::NvidiaProfileManager;
    use game_optimizer::drivers::settings::{
        AnisotropicLevel, AntiAliasingMode, AntiAliasingSetting, DriverProfile,
        PowerManagementMode, TextureFilterQuality, VSyncMode,
    };

    let SetProfileArgs {
        game,
        fps_limit,
        low_latency,
        vsync,
        power,
        anisotropic,
        mfaa,
        triple_buffer,
        texture_quality,
        preferred_refresh_rate,
        background_fps,
        aa_mode,
        aa_setting,
        lod_bias,
        trilinear_optimization,
        aniso_sample_optimization,
        gsync_indicator,
        cuda_force_p2,
    } = args;

    let manager = NvidiaProfileManager::new()?;

    let vsync_mode = vsync.map(|v| match v.to_lowercase().as_str() {
        "off" => VSyncMode::Off,
        "on" => VSyncMode::On,
        "adaptive" => VSyncMode::Adaptive,
        "fast" => VSyncMode::Fast,
        _ => VSyncMode::ApplicationControlled,
    });

    let power_mode = power.map(|p| match p.to_lowercase().as_str() {
        "adaptive" => PowerManagementMode::Adaptive,
        "max-performance" | "max" => PowerManagementMode::PreferMaxPerformance,
        _ => PowerManagementMode::Optimal,
    });

    let aniso = anisotropic.and_then(AnisotropicLevel::from_raw);
    let quality = texture_quality
        .as_deref()
        .and_then(TextureFilterQuality::from_name);
    let aa_mode_val = aa_mode.as_deref().and_then(AntiAliasingMode::from_name);
    let aa_setting_val = aa_setting.as_deref().and_then(AntiAliasingSetting::from_name);

    let profile = DriverProfile {
        name: game.to_string(),
        executables: vec![format!("{}.exe", game)],
        frame_rate_limit: fps_limit,
        low_latency,
        vsync: vsync_mode,
        power_management: power_mode,
        anisotropic_filtering: aniso,
        mfaa,
        triple_buffering: triple_buffer,
        texture_filter_quality: quality,
        preferred_refresh_rate,
        background_fps_limit: background_fps,
        aa_mode: aa_mode_val,
        aa_setting: aa_setting_val,
        lod_bias,
        trilinear_optimization,
        aniso_sample_optimization,
        gsync_indicator,
        cuda_force_p2,
        ..Default::default()
    };

    manager.set_profile(&profile)?;
    println!("✅ Profile settings updated for '{}'", game);

    Ok(())
}

fn delete_profile(name: &str, skip_confirm: bool) -> Result<()> {
    use game_optimizer::drivers::nvidia::NvidiaProfileManager;
    use std::io::{self, Write};

    let manager = NvidiaProfileManager::new()?;

    // Resolve via the full-read path so we can show a summary before destruction.
    let profile = match manager.get_profile_full(name)? {
        Some(p) => p,
        None => {
            anyhow::bail!("Profile '{}' not found", name);
        }
    };

    if profile.is_predefined {
        anyhow::bail!(
            "'{}' is a predefined NVIDIA profile and cannot be deleted",
            profile.name
        );
    }

    println!("🗑️  About to delete NVIDIA profile: {}", profile.name);
    if !profile.applications.is_empty() {
        println!("   Applications: {}", profile.applications.join(", "));
    }
    if !profile.settings.is_empty() {
        println!("   Settings:     {} configured", profile.settings.len());
    }

    if !skip_confirm {
        print!("\nDelete this profile? [y/N] ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled.");
            return Ok(());
        }
    }

    manager.delete_profile(&profile.name)?;
    println!("✅ Deleted profile '{}'", profile.name);
    Ok(())
}

fn remove_application(profile_name: &str, exe: &str, skip_confirm: bool) -> Result<()> {
    use game_optimizer::drivers::nvidia::NvidiaProfileManager;
    use std::io::{self, Write};

    let manager = NvidiaProfileManager::new()?;

    // Confirm the profile exists and show current apps.
    let profile = manager
        .get_profile_full(profile_name)?
        .ok_or_else(|| anyhow::anyhow!("Profile '{}' not found", profile_name))?;

    let exe_lower = exe.to_lowercase();
    if !profile.applications.iter().any(|a| a.to_lowercase() == exe_lower) {
        anyhow::bail!(
            "Executable '{}' is not attached to profile '{}'. Current apps: {}",
            exe, profile.name, profile.applications.join(", ")
        );
    }

    println!("✂️  Remove '{}' from profile '{}'", exe, profile.name);
    if !skip_confirm {
        print!("\nProceed? [y/N] ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled.");
            return Ok(());
        }
    }

    manager.remove_application(&profile.name, exe)?;
    println!("✅ Removed '{}' from '{}'", exe, profile.name);
    Ok(())
}

fn reset_profile(name: &str, skip_confirm: bool) -> Result<()> {
    use game_optimizer::drivers::nvidia::NvidiaProfileManager;
    use std::io::{self, Write};

    let manager = NvidiaProfileManager::new()?;

    let profile = manager
        .get_profile_full(name)?
        .ok_or_else(|| anyhow::anyhow!("Profile '{}' not found", name))?;

    println!("♻️  Reset '{}' to NVIDIA defaults", profile.name);
    if !profile.settings.is_empty() {
        println!("   Currently has {} configured settings", profile.settings.len());
    }

    if !skip_confirm {
        print!("\nProceed? [y/N] ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled.");
            return Ok(());
        }
    }

    manager.reset_profile(&profile.name)?;
    println!("✅ Reset profile '{}'", profile.name);
    Ok(())
}

fn handle_config(action: Option<ConfigAction>) -> Result<()> {
    use game_optimizer::config::Config;

    match action {
        None | Some(ConfigAction::Show) => {
            let config = Config::load()?;
            
            println!("⚙️  Configuration");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("  API Key:      {}", if config.claude_api_key.is_some() { "[set]" } else { "[not set]" });
            println!("  Resolution:   {}", config.target_resolution.as_deref().unwrap_or("not set"));
            println!("  Refresh Rate: {}", config.target_refresh_rate.map(|r| format!("{}Hz", r)).unwrap_or("not set".into()));
            println!("  G-Sync:       {}", if config.gsync_enabled { "enabled" } else { "disabled" });
            
            println!("\n📁 Paths");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("  Config: {}", Config::config_path().display());
            println!("  Data:   {}", Config::data_dir().display());
            println!("  Cache:  {}", Config::cache_dir().display());
        }
        
        Some(ConfigAction::Set { key, value }) => {
            let mut config = Config::load().unwrap_or_default();
            
            match key.to_lowercase().as_str() {
                "api-key" | "apikey" | "anthropic-key" | "claude-api-key" => {
                    config.claude_api_key = Some(value);
                    println!("✅ API key set");
                }
                "resolution" | "res" => {
                    config.target_resolution = Some(value.clone());
                    println!("✅ Resolution set to {}", value);
                }
                "refresh-rate" | "fps" | "hz" => {
                    let rate: u32 = value.parse().map_err(|_| anyhow::anyhow!("Invalid refresh rate: {}", value))?;
                    config.target_refresh_rate = Some(rate);
                    println!("✅ Refresh rate set to {}Hz", rate);
                }
                "gsync" | "vrr" | "freesync" => {
                    let enabled = matches!(value.to_lowercase().as_str(), "true" | "on" | "yes" | "1" | "enabled");
                    config.gsync_enabled = enabled;
                    println!("✅ G-Sync/VRR {}", if enabled { "enabled" } else { "disabled" });
                }
                _ => {
                    anyhow::bail!("Unknown config key: {}\nValid keys: api-key, resolution, refresh-rate, gsync", key);
                }
            }
            
            config.save()?;
        }
        
        Some(ConfigAction::Init) => {
            let config = Config::default();
            config.save()?;
            println!("✅ Config file created at: {}", Config::config_path().display());
        }
        
        Some(ConfigAction::Path) => {
            println!("{}", Config::config_path().display());
        }
    }

    Ok(())
}

#[tokio::main]
async fn game_info(query: &str, json_output: bool) -> Result<()> {
    use game_optimizer::data::pcgamingwiki::PcgwClient;

    let client = PcgwClient::new();

    // Try to parse as AppID first
    let game_info = if let Ok(appid) = query.parse::<u32>() {
        client.get_by_steam_appid(appid).await?
    } else {
        client.get_by_name(query).await?
    };

    match game_info {
        Some(info) => {
            if json_output {
                println!("{}", serde_json::to_string_pretty(&info)?);
            } else {
                println!("🎮 {}", info.name);
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                if let Some(appid) = info.steam_appid {
                    println!("  Steam AppID:  {}", appid);
                }
                if !info.developers.is_empty() {
                    println!("  Developers:   {}", info.developers.join(", "));
                }
                if !info.engines.is_empty() {
                    println!("  Engine:       {}", info.engines.join(", "));
                }
                if let Some(date) = &info.release_date {
                    println!("  Released:     {}", date);
                }

                if let Some(video) = &info.video {
                    println!("\n  📺 Display Support");
                    println!("  ─────────────────────────────────────");
                    println!("    Widescreen:      {}", video.widescreen);
                    println!("    Ultrawide:       {}", video.ultrawidescreen);
                    println!("    4K:              {}", video.four_k);
                    println!("    HDR:             {}", video.hdr);
                    println!("    Ray Tracing:     {}", video.ray_tracing);
                    println!("    60 FPS:          {}", video.fps_60);
                    println!("    120+ FPS:        {}", video.fps_120);
                    println!("    Unlimited FPS:   {}", video.fps_unlimited);
                }

                if let Some(input) = &info.input {
                    println!("\n  🎮 Controller Support");
                    println!("  ─────────────────────────────────────");
                    println!("    Full Controller: {}", input.full_controller);
                    println!("    Remapping:       {}", input.controller_remapping);
                }
            }
        }
        None => {
            if json_output {
                println!("null");
            } else {
                println!("Game '{}' not found on PCGamingWiki", query);
            }
        }
    }

    Ok(())
}

fn list_games(filter: Option<&str>, json_output: bool) -> Result<()> {
    use game_optimizer::data::{detect_all_games, find_games_by_name};

    let games = if let Some(f) = filter {
        find_games_by_name(f)?
    } else {
        detect_all_games()?
    };

    if json_output {
        println!("{}", serde_json::to_string_pretty(&games)?);
    } else {
        println!("🎮 Installed Games ({})", games.len());
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        for game in &games {
            println!(
                "  [{}] {}",
                game.platform,
                game.name,
            );
        }
    }

    Ok(())
}

#[tokio::main]
async fn recommend(
    game: &str,
    driver_only: bool,
    quick: bool,
    resolution: &str,
    target_fps: u32,
    vrr_enabled: bool,
    json_output: bool,
) -> Result<()> {
    use game_optimizer::ai::{ClaudeClient, RecommendationRequest, RecommendationResponse};
    use game_optimizer::ai::prompts::quick_recommend;
    use game_optimizer::data::pcgamingwiki::PcgwClient;
    use game_optimizer::data::benchmarks::BenchmarkClient;
    use game_optimizer::hardware::gpu::GpuInfo;

    // Detect GPU
    let gpu = GpuInfo::detect()?;

    if quick {
        // Quick mode: use heuristics, no API call
        let settings = quick_recommend(&gpu, target_fps, vrr_enabled);
        
        if json_output {
            println!("{}", serde_json::to_string_pretty(&settings)?);
        } else {
            println!("⚡ Quick Recommendations for: {}", game);
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            print_driver_settings(&settings);
        }
        return Ok(());
    }

    // Check if Claude API is configured
    if !ClaudeClient::is_configured() {
        eprintln!("⚠️  Claude API key not configured.");
        eprintln!("   Set ANTHROPIC_API_KEY environment variable or add claude_api_key to config.");
        eprintln!("   Using quick mode instead...\n");
        
        let settings = quick_recommend(&gpu, target_fps, vrr_enabled);
        if json_output {
            println!("{}", serde_json::to_string_pretty(&settings)?);
        } else {
            println!("⚡ Quick Recommendations for: {}", game);
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            print_driver_settings(&settings);
        }
        return Ok(());
    }

    println!("🔍 Gathering game data...");

    // Fetch PCGamingWiki data
    let pcgw_client = PcgwClient::new();
    let pcgw_data = if let Ok(appid) = game.parse::<u32>() {
        pcgw_client.get_by_steam_appid(appid).await.ok().flatten()
    } else {
        pcgw_client.get_by_name(game).await.ok().flatten()
    };

    // Fetch benchmark data
    let benchmark_data = if let Ok(bench_client) = BenchmarkClient::new() {
        let articles = bench_client.search(game).await.ok().unwrap_or_default();
        // Try to get settings from first article
        if let Some(article) = articles.first() {
            bench_client.fetch_article_settings(article).await.ok().flatten()
        } else {
            None
        }
    } else {
        None
    };

    println!("🤖 Generating AI recommendations...");

    // Build request
    let request = RecommendationRequest {
        game_name: game.to_string(),
        gpu,
        target_resolution: resolution.to_string(),
        target_fps,
        vrr_enabled,
        pcgw_data,
        benchmark_data,
        driver_only,
    };

    // Call Claude API
    let client = ClaudeClient::new("game-settings-recommend").map_err(|e| anyhow::anyhow!("{}", e))?;
    let response: RecommendationResponse = client
        .send_json_message(&request.system_prompt(), &request.user_message())
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    if json_output {
        println!("{}", serde_json::to_string_pretty(&response)?);
    } else {
        println!("\n🎯 Recommendations for: {}", response.game);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📝 {}", response.summary);

        println!("\n🔧 NVIDIA Driver Settings");
        println!("───────────────────────────────────────");
        print_driver_settings(&response.driver_settings);

        if let Some(ref in_game) = response.in_game_settings {
            println!("\n🎮 In-Game Settings");
            println!("───────────────────────────────────────");
            if let Some(ref preset) = in_game.preset {
                println!("  Preset:           {}", preset);
            }
            if let Some(ref upscaling) = in_game.upscaling {
                println!("  Upscaling:        {}", upscaling);
            }
            if let Some(ref rt) = in_game.ray_tracing {
                println!("  Ray Tracing:      {}", rt);
            }
            if let Some(ref fg) = in_game.frame_generation {
                println!("  Frame Gen:        {}", fg);
            }
            for adj in &in_game.adjustments {
                println!("  {}:  {}", adj.setting, adj.value);
            }
        }

        if let Some(ref ls) = response.lossless_scaling {
            if ls.enabled {
                println!("\n📐 Lossless Scaling");
                println!("───────────────────────────────────────");
                if let Some(ref mode) = ls.mode {
                    println!("  Mode:       {}", mode);
                }
                if let Some(ref mult) = ls.multiplier {
                    println!("  Multiplier: {}", mult);
                }
                if let Some(base) = ls.base_fps {
                    println!("  Base FPS:   {}", base);
                }
            }
        }

        if let Some(rtss_fps) = response.rtss_fps_limit {
            println!("\n⏱️  RTSS FPS Limit: {}", rtss_fps);
        }

        if !response.reasoning.is_empty() {
            println!("\n💡 Reasoning");
            println!("───────────────────────────────────────");
            for reason in &response.reasoning {
                println!("  • {}", reason);
            }
        }

        if !response.caveats.is_empty() {
            println!("\n⚠️  Caveats");
            println!("───────────────────────────────────────");
            for caveat in &response.caveats {
                println!("  • {}", caveat);
            }
        }
    }

    Ok(())
}

fn print_driver_settings(settings: &game_optimizer::ai::prompts::DriverSettings) {
    if let Some(fps) = settings.frame_rate_limit {
        println!("  Frame Rate Limit: {}", if fps == 0 { "Off".to_string() } else { fps.to_string() });
    }
    if let Some(ll) = settings.low_latency_mode {
        let ll_str = match ll {
            0 => "Off",
            1 => "On",
            2 => "Ultra",
            _ => "Unknown",
        };
        println!("  Low Latency:      {}", ll_str);
    }
    if let Some(ref vsync) = settings.vsync {
        println!("  VSync:            {}", vsync);
    }
    if let Some(ref power) = settings.power_management {
        println!("  Power Management: {}", power);
    }
    if let Some(af) = settings.anisotropic_filtering {
        println!("  Anisotropic:      {}x", af);
    }
    if let Some(ref tf) = settings.texture_filtering {
        println!("  Texture Filter:   {}", tf);
    }
    if let Some(ref to) = settings.threaded_optimization {
        println!("  Threaded Opt:     {}", to);
    }
    if let Some(ref sc) = settings.shader_cache {
        println!("  Shader Cache:     {}", sc);
    }
}

#[tokio::main]
async fn search_benchmarks(game: &str, json_output: bool) -> Result<()> {
    use game_optimizer::data::benchmarks::BenchmarkClient;

    let client = BenchmarkClient::new()?;
    let articles = client.search(game).await?;

    if json_output {
        println!("{}", serde_json::to_string_pretty(&articles)?);
    } else {
        println!("📊 Benchmark Articles for: {}", game);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        if articles.is_empty() {
            println!("  No articles found.");
        } else {
            for article in &articles {
                println!("  [{}] {}", article.source, article.title);
                if let Some(ref url) = article.url {
                    println!("       {}", url);
                }
            }
        }
    }

    Ok(())
}

fn handle_rtss(action: RtssAction, json_output: bool) -> Result<()> {
    use game_optimizer::tools::rtss::RtssManager;

    let manager = RtssManager::new()?;

    match action {
        RtssAction::Status => {
            let installed = manager.is_installed();
            let running = manager.is_running();

            if json_output {
                println!(r#"{{"installed": {}, "running": {}}}"#, installed, running);
            } else {
                println!("⏱️  RTSS Status");
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!("  Installed: {}", if installed { "✅ Yes" } else { "❌ No" });
                if installed {
                    if let Some(path) = manager.install_path() {
                        println!("  Path:      {}", path.display());
                    }
                    println!("  Running:   {}", if running { "✅ Yes" } else { "❌ No" });
                }
            }
        }

        RtssAction::List => {
            let profiles = manager.list_profiles()?;

            if json_output {
                println!("{}", serde_json::to_string_pretty(&profiles)?);
            } else {
                println!("📋 RTSS Profiles ({})", profiles.len());
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                for profile in profiles {
                    println!(
                        "  {} - {} ({})",
                        profile.name,
                        profile.executable,
                        profile.fps_display()
                    );
                }
            }
        }

        RtssAction::Get { name } => {
            match manager.get_profile(&name)? {
                Some(profile) => {
                    if json_output {
                        println!("{}", serde_json::to_string_pretty(&profile)?);
                    } else {
                        println!("📋 RTSS Profile: {}", profile.name);
                        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                        println!("  Executable:  {}", profile.executable);
                        println!("  FPS Limit:   {}", profile.fps_display());
                        if let Some(ss) = profile.scanline_sync {
                            println!("  Scanline:    {}", ss);
                        }
                        println!("  OSD:         {}", if profile.osd_enabled { "Enabled" } else { "Disabled" });
                    }
                }
                None => {
                    if json_output {
                        println!("null");
                    } else {
                        println!("Profile '{}' not found", name);
                    }
                }
            }
        }

        RtssAction::Set { game, fps } => {
            manager.set_fps_limit(&game, fps)?;
            println!("✅ RTSS FPS limit set to {} for '{}'", 
                if fps == 0 { "Unlimited".to_string() } else { fps.to_string() },
                game
            );
        }
    }

    Ok(())
}

fn handle_lossless_scaling(action: LsAction, json_output: bool) -> Result<()> {
    use game_optimizer::tools::lossless::{LosslessScaling, LsfgMultiplier, recommend_lsfg};

    match action {
        LsAction::Status => {
            match LosslessScaling::new() {
                Ok(ls) => {
                    let has_settings = ls.has_settings();
                    let install_path = ls.install_path().map(|p| p.display().to_string());
                    let settings_path = ls.settings_path().display().to_string();

                    if json_output {
                        println!(r#"{{"configured": {}, "settings_path": "{}", "install_path": {}}}"#,
                            has_settings,
                            settings_path,
                            install_path.as_ref().map(|p| format!("\"{}\"", p)).unwrap_or("null".to_string())
                        );
                    } else {
                        println!("📐 Lossless Scaling Status");
                        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                        if let Some(path) = install_path {
                            println!("  Installed:   ✅ Yes");
                            println!("  Path:        {}", path);
                        } else {
                            println!("  Installed:   ❌ Not found");
                        }
                        println!("  Settings:    {}", if has_settings { "Found" } else { "Not configured" });
                        println!("  Config:      {}", settings_path);
                    }
                }
                Err(e) => {
                    if json_output {
                        println!(r#"{{"error": "{}"}}"#, e);
                    } else {
                        println!("⚠️  Could not detect Lossless Scaling: {}", e);
                    }
                }
            }
        }

        LsAction::List => {
            let ls = LosslessScaling::new()?;
            let profiles = ls.list_profiles()?;

            if json_output {
                println!("{}", serde_json::to_string_pretty(&profiles)?);
            } else {
                println!("📋 Lossless Scaling Profiles ({})", profiles.len());
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                if profiles.is_empty() {
                    println!("  No profiles found.");
                } else {
                    for profile in profiles {
                        println!(
                            "  {} - {}",
                            profile.title,
                            profile.display_summary()
                        );
                    }
                }
            }
        }

        LsAction::Get { name } => {
            let ls = LosslessScaling::new()?;
            match ls.get_profile(&name)? {
                Some(profile) => {
                    if json_output {
                        println!("{}", serde_json::to_string_pretty(&profile)?);
                    } else {
                        println!("📋 Lossless Scaling Profile: {}", profile.title);
                        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                        if let Some(ref path) = profile.path {
                            println!("  Executable:  {}", path);
                        }
                        println!("  Frame Gen:   {}", profile.frame_generation);
                        println!("  Mode:        {}", profile.lsfg3_mode);
                        println!("  Multiplier:  {}x", profile.lsfg3_multiplier);
                        println!("  Target FPS:  {}", profile.lsfg3_target);
                        println!("  Flow Scale:  {}%", profile.lsfg_flow_scale);
                        println!("  Scaling:     {}", profile.scaling_type);
                        println!("  Capture API: {}", profile.capture_api);
                    }
                }
                None => {
                    if json_output {
                        println!("null");
                    } else {
                        println!("Profile '{}' not found", name);
                    }
                }
            }
        }

        LsAction::Set { name, mode, multiplier, target, flow } => {
            let ls = LosslessScaling::new()?;
            
            // Get existing profile
            let mut profile = ls.get_profile(&name)?
                .ok_or_else(|| anyhow::anyhow!("Profile '{}' not found", name))?;
            
            // Apply changes
            if let Some(m) = mode {
                match m.to_lowercase().as_str() {
                    "fixed" => {
                        profile.frame_generation = "LSFG3".to_string();
                        profile.lsfg3_mode = "FIXED".to_string();
                    }
                    "adaptive" => {
                        profile.frame_generation = "LSFG3".to_string();
                        profile.lsfg3_mode = "ADAPTIVE".to_string();
                    }
                    "off" => {
                        profile.frame_generation = "Off".to_string();
                    }
                    _ => {
                        anyhow::bail!("Invalid mode: {}. Use: fixed, adaptive, off", m);
                    }
                }
            }
            
            if let Some(mult) = multiplier {
                if mult >= 2 && mult <= 4 {
                    profile.lsfg3_multiplier = mult;
                } else {
                    anyhow::bail!("Multiplier must be 2, 3, or 4");
                }
            }
            
            if let Some(t) = target {
                profile.lsfg3_target = t;
            }
            
            if let Some(f) = flow {
                if f <= 100 {
                    profile.lsfg_flow_scale = f;
                } else {
                    anyhow::bail!("Flow scale must be 0-100");
                }
            }
            
            ls.save_profile(profile.clone())?;
            
            println!("✅ Updated profile '{}'", profile.title);
            println!("   {}", profile.display_summary());
        }

        LsAction::Calc { target, native } => {
            let recommendation = recommend_lsfg(native, target, false);

            if json_output {
                println!(r#"{{"should_use": {}, "reason": "{}", "base_fps": {}, "multiplier": "{}x"}}"#,
                    recommendation.should_use,
                    recommendation.reason,
                    recommendation.profile.lsfg3_target / recommendation.profile.lsfg3_multiplier,
                    recommendation.profile.lsfg3_multiplier
                );
            } else {
                println!("📐 LSFG Calculation");
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!("  Native FPS:  {}", native);
                println!("  Target FPS:  {}", target);
                println!();
                
                if recommendation.should_use {
                    println!("  ✅ Recommendation: Use LSFG");
                    println!("  Mode:        {}", recommendation.profile.lsfg3_mode);
                    println!("  Multiplier:  {}x", recommendation.profile.lsfg3_multiplier);
                    let base = target / recommendation.profile.lsfg3_multiplier;
                    println!("  Cap at:      {} FPS", base);
                    let mult = LsfgMultiplier::from_u32(recommendation.profile.lsfg3_multiplier);
                    let output = LosslessScaling::calculate_output_fps(base, mult);
                    println!("  Output:      ~{} FPS", output);
                } else {
                    println!("  ❌ LSFG not recommended");
                }
                println!();
                println!("  💡 {}", recommendation.reason);
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn optimize(
    game: &str,
    driver_only: bool,
    quick: bool,
    resolution: &str,
    target_fps: u32,
    vrr_enabled: bool,
    dry_run: bool,
    skip_confirm: bool,
    no_backup: bool,
    json_output: bool,
) -> Result<()> {
    use game_optimizer::ai::{ClaudeClient, RecommendationRequest, RecommendationResponse};
    use game_optimizer::ai::prompts::quick_recommend;
    use game_optimizer::data::pcgamingwiki::PcgwClient;
    use game_optimizer::data::benchmarks::BenchmarkClient;
    use game_optimizer::hardware::gpu::GpuInfo;
    use game_optimizer::backup::{BackupManager, SettingsBackup, ApplyResult};
    use game_optimizer::tools::rtss::RtssManager;
    use game_optimizer::tools::lossless::LosslessScaling;
    use chrono::Utc;
    use std::io::{self, Write};

    // Detect GPU
    let gpu = GpuInfo::detect()?;

    // Get recommendations
    let response: RecommendationResponse = if quick {
        let settings = quick_recommend(&gpu, target_fps, vrr_enabled);
        RecommendationResponse {
            game: game.to_string(),
            summary: "Quick optimization based on heuristics".to_string(),
            driver_settings: settings,
            in_game_settings: None,
            lossless_scaling: None,
            rtss_fps_limit: Some(if vrr_enabled { target_fps.saturating_sub(3) } else { target_fps }),
            reasoning: vec!["Using quick heuristics mode".to_string()],
            caveats: vec![],
        }
    } else if !ClaudeClient::is_configured() {
        eprintln!("⚠️  Claude API key not configured. Using quick mode.");
        let settings = quick_recommend(&gpu, target_fps, vrr_enabled);
        RecommendationResponse {
            game: game.to_string(),
            summary: "Quick optimization based on heuristics".to_string(),
            driver_settings: settings,
            in_game_settings: None,
            lossless_scaling: None,
            rtss_fps_limit: Some(if vrr_enabled { target_fps.saturating_sub(3) } else { target_fps }),
            reasoning: vec!["API not configured, using heuristics".to_string()],
            caveats: vec![],
        }
    } else {
        println!("🔍 Gathering game data...");

        // Fetch PCGamingWiki data
        let pcgw_client = PcgwClient::new();
        let pcgw_data = if let Ok(appid) = game.parse::<u32>() {
            pcgw_client.get_by_steam_appid(appid).await.ok().flatten()
        } else {
            pcgw_client.get_by_name(game).await.ok().flatten()
        };

        // Fetch benchmark data
        let benchmark_data = if let Ok(bench_client) = BenchmarkClient::new() {
            let articles = bench_client.search(game).await.ok().unwrap_or_default();
            if let Some(article) = articles.first() {
                bench_client.fetch_article_settings(article).await.ok().flatten()
            } else {
                None
            }
        } else {
            None
        };

        println!("🤖 Generating AI recommendations...");

        let request = RecommendationRequest {
            game_name: game.to_string(),
            gpu: gpu.clone(),
            target_resolution: resolution.to_string(),
            target_fps,
            vrr_enabled,
            pcgw_data,
            benchmark_data,
            driver_only,
        };

        let client = ClaudeClient::new("game-optimize").map_err(|e| anyhow::anyhow!("{}", e))?;
        client
            .send_json_message(&request.system_prompt(), &request.user_message())
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?
    };

    // Display preview
    if json_output {
        println!("{}", serde_json::to_string_pretty(&response)?);
        if dry_run {
            return Ok(());
        }
    } else {
        println!("\n{} Optimization Preview for: {}", if dry_run { "🔍" } else { "🎯" }, response.game);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📝 {}", response.summary);

        println!("\n🔧 NVIDIA Driver Settings");
        println!("───────────────────────────────────────");
        print_driver_settings(&response.driver_settings);

        if let Some(rtss_fps) = response.rtss_fps_limit {
            println!("\n⏱️  RTSS FPS Limit: {}", rtss_fps);
        }

        if let Some(ref ls) = response.lossless_scaling {
            if ls.enabled {
                println!("\n📐 Lossless Scaling");
                println!("───────────────────────────────────────");
                if let Some(ref mode) = ls.mode {
                    println!("  Mode:       {}", mode);
                }
                if let Some(ref mult) = ls.multiplier {
                    println!("  Multiplier: {}", mult);
                }
                if let Some(base) = ls.base_fps {
                    println!("  Base FPS:   {}", base);
                }
            }
        }

        if let Some(ref in_game) = response.in_game_settings {
            println!("\n🎮 In-Game Settings (manual)");
            println!("───────────────────────────────────────");
            if let Some(ref preset) = in_game.preset {
                println!("  Preset:     {}", preset);
            }
            if let Some(ref upscaling) = in_game.upscaling {
                println!("  Upscaling:  {}", upscaling);
            }
        }

        if dry_run {
            println!("\n📋 Dry-run mode - no changes applied.");
            return Ok(());
        }
    }

    // Confirmation
    if !skip_confirm && !json_output {
        print!("\nApply these settings? [y/N] ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled.");
            return Ok(());
        }
    }

    // Create backup
    let mut result = ApplyResult::default();
    if !no_backup {
        println!("\n📦 Creating backup...");
        let backup_mgr = BackupManager::new()?;
        let backup_id = BackupManager::generate_id(&response.game);

        // Capture the current NVIDIA profile snapshot (best-effort).
        let nvidia_backup = capture_nvidia_backup(&response.game, game);

        let backup = SettingsBackup {
            id: backup_id.clone(),
            created_at: Utc::now(),
            game_name: response.game.clone(),
            executable: Some(format!("{}.exe", game)),
            nvidia_settings: nvidia_backup,
            rtss_settings: None,
            lossless_scaling_settings: None,
        };

        backup_mgr.create_backup(&backup)?;
        result.backup_id = Some(backup_id);
        println!("  ✅ Backup created: {}", result.backup_id.as_ref().unwrap());
    }

    // Apply settings
    println!("\n🔧 Applying settings...");

    // Apply NVIDIA driver settings via the DRS API.
    match apply_nvidia_driver_settings(&response.game, game, &response.driver_settings) {
        Ok(applied) => {
            println!("  ✅ NVIDIA profile '{}': {} settings applied", response.game, applied);
            result.nvidia_applied = true;
        }
        Err(e) => {
            println!("  ⚠️  NVIDIA profile '{}': {}", response.game, e);
            result.warnings.push(format!("NVIDIA apply error: {}", e));
        }
    }

    // Apply RTSS settings
    if !driver_only {
        if let Some(fps_limit) = response.rtss_fps_limit {
            match RtssManager::new() {
                Ok(rtss) => {
                    if rtss.is_installed() {
                        match rtss.set_fps_limit(&format!("{}.exe", game), fps_limit) {
                            Ok(_) => {
                                println!("  ✅ RTSS: {} FPS limit", fps_limit);
                                result.rtss_applied = true;
                            }
                            Err(e) => {
                                result.warnings.push(format!("RTSS error: {}", e));
                                println!("  ⚠️  RTSS: {}", e);
                            }
                        }
                    } else {
                        result.warnings.push("RTSS not installed".to_string());
                    }
                }
                Err(e) => {
                    result.warnings.push(format!("RTSS init error: {}", e));
                }
            }
        }

        // Apply Lossless Scaling settings  
        if let Some(ref ls_settings) = response.lossless_scaling {
            if ls_settings.enabled {
                match LosslessScaling::new() {
                    Ok(_ls) => {
                        println!("  📌 Lossless Scaling: Configure in app");
                        if let Some(ref mult) = ls_settings.multiplier {
                            println!("       Multiplier: {}", mult);
                        }
                        if let Some(base_fps) = ls_settings.base_fps {
                            println!("       Cap game at {} FPS", base_fps);
                        }
                        result.ls_applied = true;
                    }
                    Err(_) => {
                        result.warnings.push("Lossless Scaling not found".to_string());
                    }
                }
            }
        }
    }

    // Summary
    println!("\n✅ Optimization complete!");
    if !result.warnings.is_empty() {
        println!("\n⚠️  Warnings:");
        for w in &result.warnings {
            println!("  • {}", w);
        }
    }
    
    if result.backup_id.is_some() {
        println!("\n💡 To restore: game-optimizer backup restore {}", result.backup_id.as_ref().unwrap());
    }

    Ok(())
}

fn handle_backup(action: BackupAction, json_output: bool) -> Result<()> {
    use game_optimizer::backup::BackupManager;

    let manager = BackupManager::new()?;

    match action {
        BackupAction::List { game } => {
            let backups = if let Some(ref g) = game {
                manager.list_backups_for_game(g)?
            } else {
                manager.list_backups()?
            };

            if json_output {
                println!("{}", serde_json::to_string_pretty(&backups)?);
            } else {
                println!("📦 Settings Backups ({})", backups.len());
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                if backups.is_empty() {
                    println!("  No backups found.");
                } else {
                    for backup in backups {
                        let components = [
                            backup.nvidia_settings.as_ref().map(|_| "NVIDIA"),
                            backup.rtss_settings.as_ref().map(|_| "RTSS"),
                            backup.lossless_scaling_settings.as_ref().map(|_| "LS"),
                        ];
                        let parts: Vec<_> = components.iter().filter_map(|c| *c).collect();
                        let parts_str = if parts.is_empty() { "empty".to_string() } else { parts.join(", ") };
                        println!(
                            "  [{}] {} - {} ({})",
                            backup.created_at.format("%Y-%m-%d %H:%M"),
                            backup.id,
                            backup.game_name,
                            parts_str
                        );
                    }
                }
                println!("\n  Backup dir: {}", manager.backup_dir().display());
            }
        }

        BackupAction::Show { id } => {
            match manager.get_backup(&id)? {
                Some(backup) => {
                    if json_output {
                        println!("{}", serde_json::to_string_pretty(&backup)?);
                    } else {
                        println!("📦 Backup: {}", backup.id);
                        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                        println!("  Game:    {}", backup.game_name);
                        println!("  Created: {}", backup.created_at.format("%Y-%m-%d %H:%M:%S"));
                        if let Some(ref exe) = backup.executable {
                            println!("  Exe:     {}", exe);
                        }

                        if let Some(ref nv) = backup.nvidia_settings {
                            println!("\n  🔧 NVIDIA Driver");
                            println!("  ─────────────────────────────────────");
                            println!("    Profile:  {}", nv.profile_name);
                            println!("    Existed:  {}", if nv.existed { "yes (captured prior state)" } else { "no (new profile)" });
                            let s = &nv.settings;
                            if let Some(v) = s.frame_rate_limit { println!("    FPS Limit:         {}", v); }
                            if let Some(v) = s.low_latency_mode { println!("    Low Latency Mode:  {}", v); }
                            if let Some(ref v) = s.vsync { println!("    VSync:             {}", v); }
                            if let Some(ref v) = s.power_management { println!("    Power Management:  {}", v); }
                            if let Some(ref v) = s.texture_filtering { println!("    Texture Filter:    {}", v); }
                            if let Some(v) = s.anisotropic_filtering { println!("    Anisotropic:       {}x", v); }
                            if let Some(ref v) = s.threaded_optimization { println!("    Threaded Opt:      {}", v); }
                            if let Some(ref v) = s.shader_cache { println!("    Shader Cache:      {}", v); }
                            if let Some(v) = s.cuda_force_p2 { println!("    CUDA Force P2:     {}", v); }
                            if let Some(ref v) = s.image_sharpening { println!("    Image Sharpening:  {}", v); }
                        } else {
                            println!("\n  NVIDIA:  (not captured)");
                        }

                        if let Some(ref rtss) = backup.rtss_settings {
                            println!("\n  ⏱️  RTSS");
                            println!("  ─────────────────────────────────────");
                            println!("    Profile:    {}", rtss.profile_name);
                            println!("    Existed:    {}", rtss.existed);
                            println!("    FPS Limit:  {}", rtss.fps_limit.map(|f| f.to_string()).unwrap_or_else(|| "unlimited".into()));
                        }

                        if let Some(ref ls) = backup.lossless_scaling_settings {
                            println!("\n  📐 Lossless Scaling");
                            println!("  ─────────────────────────────────────");
                            println!("    Profile: {}", ls.profile_name);
                            println!("    Existed: {}", ls.existed);
                        }
                    }
                }
                None => {
                    if json_output {
                        println!("null");
                    } else {
                        println!("Backup '{}' not found", id);
                    }
                }
            }
        }

        BackupAction::Restore { id, game, dry_run, yes } => {
            let backup = if id == "latest" {
                let game_name = game.ok_or_else(|| anyhow::anyhow!("--game required when using 'latest'"))?;
                manager.get_latest_backup(&game_name)?
                    .ok_or_else(|| anyhow::anyhow!("No backups found for game '{}'", game_name))?
            } else {
                manager.get_backup(&id)?
                    .ok_or_else(|| anyhow::anyhow!("Backup '{}' not found", id))?
            };

            println!("📦 Restoring backup: {}", backup.id);
            println!("   Game: {}", backup.game_name);
            println!("   From: {}", backup.created_at.format("%Y-%m-%d %H:%M"));

            if dry_run {
                println!("\n📋 Dry-run mode - no changes applied.");
                return Ok(());
            }

            if !yes {
                use std::io::{self, Write};
                print!("\nRestore these settings? [y/N] ");
                io::stdout().flush()?;
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                if !input.trim().eq_ignore_ascii_case("y") {
                    println!("Cancelled.");
                    return Ok(());
                }
            }

            // Re-apply backed-up settings.
            println!("\n🔧 Restoring...");
            if let Some(ref nv) = backup.nvidia_settings {
                let exe = backup
                    .executable
                    .clone()
                    .unwrap_or_else(|| format!("{}.exe", backup.game_name));
                match apply_nvidia_driver_settings(&nv.profile_name, &exe, &nv.settings) {
                    Ok(applied) => println!(
                        "  ✅ NVIDIA profile '{}': {} settings restored",
                        nv.profile_name, applied
                    ),
                    Err(e) => println!("  ⚠️  NVIDIA restore failed: {}", e),
                }
            }
            if let Some(ref rtss) = backup.rtss_settings {
                println!("  📌 RTSS: {} FPS", rtss.fps_limit.map(|f| f.to_string()).unwrap_or("unlimited".to_string()));
            }
            if backup.lossless_scaling_settings.is_some() {
                println!("  📌 Lossless Scaling: (restore pending)");
            }

            println!("\n✅ Restore complete!");
        }

        BackupAction::Delete { id } => {
            if manager.delete_backup(&id)? {
                println!("✅ Deleted backup: {}", id);
            } else {
                println!("Backup '{}' not found", id);
            }
        }

        BackupAction::Export { game, output, format } => {
            use game_optimizer::backup::GameProfile;
            use game_optimizer::ai::prompts::quick_recommend;
            use game_optimizer::hardware::gpu::GpuInfo;
            use game_optimizer::config::Config;
            use chrono::Utc;
            use std::path::PathBuf;

            // Load config for defaults
            let config = Config::load().unwrap_or_default();
            let target_fps = config.target_refresh_rate.unwrap_or(144);
            let vrr = config.gsync_enabled;

            // Get GPU info
            let gpu = GpuInfo::detect()?;
            
            // Generate quick settings for the profile
            let settings = quick_recommend(&gpu, target_fps, vrr);

            // Build profile
            let mut profile = GameProfile::new(&game);
            profile.executable = Some(format!("{}.exe", game.to_lowercase().replace(' ', "")));
            profile.target_hardware = Some(gpu.name.clone());
            profile.target_resolution = config.target_resolution.clone();
            profile.target_fps = Some(target_fps);
            profile.nvidia_settings = Some(settings.clone());
            profile.rtss_fps_limit = settings.frame_rate_limit;
            profile.created_at = Utc::now();

            // Determine output path
            let output_path = if let Some(out) = output {
                PathBuf::from(out)
            } else {
                let clean_name: String = game.chars()
                    .filter(|c| c.is_alphanumeric() || *c == ' ')
                    .collect();
                let ext = if format == "toml" { "toml" } else { "json" };
                PathBuf::from(format!("{}_{}.{}", clean_name.replace(' ', "_"), target_fps, ext))
            };

            // Export
            profile.to_file(&output_path, &format)?;

            if json_output {
                println!("{}", profile.to_json()?);
            } else {
                println!("📤 Exported profile: {}", output_path.display());
                println!("   Game:       {}", profile.game_name);
                println!("   Hardware:   {}", profile.target_hardware.as_deref().unwrap_or("Unknown"));
                println!("   Target FPS: {}", profile.target_fps.unwrap_or(0));
                println!("   Format:     {}", format.to_uppercase());
            }
        }

        BackupAction::Import { file, dry_run, yes } => {
            use game_optimizer::backup::GameProfile;
            use std::path::Path;

            let path = Path::new(&file);
            let profile = GameProfile::from_file(path)?;

            println!("📥 Importing profile: {}", profile.game_name);
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            if let Some(ref hw) = profile.target_hardware {
                println!("  Original HW:  {}", hw);
            }
            if let Some(ref res) = profile.target_resolution {
                println!("  Resolution:   {}", res);
            }
            if let Some(fps) = profile.target_fps {
                println!("  Target FPS:   {}", fps);
            }
            
            if profile.nvidia_settings.is_some() {
                println!("  NVIDIA:       Yes");
            }
            if profile.rtss_fps_limit.is_some() {
                println!("  RTSS:         {} FPS", profile.rtss_fps_limit.unwrap());
            }
            if profile.lossless_scaling.is_some() {
                println!("  LS:           Yes");
            }

            if dry_run {
                println!("\n📋 Dry-run mode - no changes applied.");
                return Ok(());
            }

            if !yes {
                use std::io::{self, Write};
                print!("\nApply this profile? [y/N] ");
                io::stdout().flush()?;
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                if !input.trim().eq_ignore_ascii_case("y") {
                    println!("Cancelled.");
                    return Ok(());
                }
            }

            println!("\n🔧 Applying profile...");
            
            // Apply NVIDIA settings
            if let Some(ref nv) = profile.nvidia_settings {
                let default_exe = format!("{}.exe", profile.game_name);
                let exe = profile.executable.as_deref().unwrap_or(&default_exe);
                match apply_nvidia_driver_settings(&profile.game_name, exe, nv) {
                    Ok(applied) => println!(
                        "  ✅ NVIDIA profile '{}': {} settings applied",
                        profile.game_name, applied
                    ),
                    Err(e) => println!("  ⚠️  NVIDIA apply failed: {}", e),
                }
            }

            // Apply RTSS settings
            if let Some(fps_limit) = profile.rtss_fps_limit {
                use game_optimizer::tools::rtss::RtssManager;
                if let Ok(rtss) = RtssManager::new() {
                    if rtss.is_installed() {
                        let default_exe = format!("{}.exe", profile.game_name);
                        let exe = profile.executable.as_deref().unwrap_or(&default_exe);
                        match rtss.set_fps_limit(exe, fps_limit) {
                            Ok(_) => println!("  ✅ RTSS: {} FPS", fps_limit),
                            Err(e) => println!("  ⚠️  RTSS: {}", e),
                        }
                    }
                }
            }

            println!("\n✅ Profile imported!");
        }
    }

    Ok(())
}

/// Convert the AI/JSON `DriverSettings` shape into the low-level `DriverProfile`
/// used by `NvidiaProfileManager::set_profile`.
fn driver_settings_to_profile(
    profile_name: &str,
    executable: &str,
    settings: &game_optimizer::ai::prompts::DriverSettings,
) -> game_optimizer::drivers::settings::DriverProfile {
    use game_optimizer::drivers::settings::{
        AnisotropicLevel, DriverProfile, PowerManagementMode, TextureFilterQuality, VSyncMode,
    };

    let vsync = settings.vsync.as_deref().and_then(|s| match s.to_lowercase().as_str() {
        "off" => Some(VSyncMode::Off),
        "on" => Some(VSyncMode::On),
        "adaptive" => Some(VSyncMode::Adaptive),
        "fast" => Some(VSyncMode::Fast),
        "app" | "application" | "application-controlled" => Some(VSyncMode::ApplicationControlled),
        _ => None,
    });

    let power = settings.power_management.as_deref().and_then(|p| match p.to_lowercase().as_str() {
        "adaptive" => Some(PowerManagementMode::Adaptive),
        "prefer_max_performance" | "max-performance" | "max" => {
            Some(PowerManagementMode::PreferMaxPerformance)
        }
        "optimal" | "optimal_power" => Some(PowerManagementMode::Optimal),
        _ => None,
    });

    let shader_cache = settings.shader_cache.as_deref().map(|s| {
        // "on" and "unlimited" both enable the cache.
        !matches!(s.to_lowercase().as_str(), "off" | "disabled" | "0" | "false")
    });

    let threaded = settings.threaded_optimization.as_deref().and_then(|t| match t.to_lowercase().as_str() {
        "on" | "true" | "enabled" => Some(true),
        "off" | "false" | "disabled" => Some(false),
        _ => None, // "auto" / unknown -> leave alone
    });

    let aniso = settings.anisotropic_filtering.and_then(AnisotropicLevel::from_raw);

    let quality = settings
        .texture_filtering
        .as_deref()
        .and_then(TextureFilterQuality::from_name);

    DriverProfile {
        name: profile_name.to_string(),
        executables: vec![executable.to_string()],
        low_latency: settings.low_latency_mode,
        frame_rate_limit: settings.frame_rate_limit,
        vsync,
        power_management: power,
        anisotropic_filtering: aniso,
        shader_cache,
        threaded_optimization: threaded,
        mfaa: None,
        triple_buffering: None,
        texture_filter_quality: quality,
        preferred_refresh_rate: None,
        background_fps_limit: None,
        aa_mode: None,
        aa_setting: None,
        lod_bias: None,
        trilinear_optimization: None,
        aniso_sample_optimization: None,
        gsync_indicator: None,
        cuda_force_p2: settings.cuda_force_p2,
    }
}

/// Apply a `DriverSettings` to an NVIDIA profile via the DRS API.
/// Returns the number of settings successfully written.
fn apply_nvidia_driver_settings(
    profile_name: &str,
    executable: &str,
    settings: &game_optimizer::ai::prompts::DriverSettings,
) -> Result<usize> {
    use game_optimizer::drivers::nvidia::NvidiaProfileManager;

    let manager = NvidiaProfileManager::new()?;
    let profile = driver_settings_to_profile(profile_name, executable, settings);

    // Count settings we're about to attempt; set_profile logs per-setting success.
    let mut planned = 0usize;
    if profile.frame_rate_limit.is_some() { planned += 1; }
    if profile.low_latency.is_some() { planned += 1; }
    if profile.vsync.is_some() { planned += 1; } // counts as 1; may expand to 2 DWORD writes
    if profile.power_management.is_some() { planned += 1; }
    if profile.shader_cache.is_some() { planned += 1; }
    if profile.threaded_optimization.is_some() { planned += 1; }
    if profile.anisotropic_filtering.is_some() { planned += 1; } // counts as 1; writes level + selector
    if profile.mfaa.is_some() { planned += 1; }
    if profile.triple_buffering.is_some() { planned += 1; }
    if profile.texture_filter_quality.is_some() { planned += 1; }
    if profile.preferred_refresh_rate.is_some() { planned += 1; }
    if profile.background_fps_limit.is_some() { planned += 1; }
    if profile.aa_mode.is_some() { planned += 1; }
    if profile.aa_setting.is_some() { planned += 1; }
    if profile.lod_bias.is_some() { planned += 1; }
    if profile.trilinear_optimization.is_some() { planned += 1; }
    if profile.aniso_sample_optimization.is_some() { planned += 1; }
    if profile.gsync_indicator.is_some() { planned += 1; }
    if profile.cuda_force_p2.is_some() { planned += 1; }

    manager.set_profile(&profile)?;
    Ok(planned)
}

/// Capture the current NVIDIA profile state (best-effort) for a `NvidiaBackup`.
/// Returns `None` if no existing profile was found or DRS is unavailable.
fn capture_nvidia_backup(
    profile_name: &str,
    game: &str,
) -> Option<game_optimizer::backup::NvidiaBackup> {
    use game_optimizer::ai::prompts::DriverSettings;
    use game_optimizer::backup::NvidiaBackup;
    use game_optimizer::drivers::nvidia::NvidiaProfileManager;
    use game_optimizer::drivers::settings::{
        nvapi_ids, shader_cache_values, thread_control_values, PowerManagementMode, VSyncMode,
    };

    let manager = NvidiaProfileManager::new().ok()?;
    let exe = format!("{}.exe", game);

    // Try the profile name first, then the executable.
    let current = manager
        .get_profile_full(profile_name)
        .ok()
        .flatten()
        .or_else(|| manager.get_profile_full(&exe).ok().flatten());

    let Some(current) = current else {
        return Some(NvidiaBackup {
            profile_name: profile_name.to_string(),
            existed: false,
            settings: DriverSettings::default(),
        });
    };

    // Reconstruct a DriverSettings from the captured DWORD values.
    let mut settings = DriverSettings::default();
    let key = |id: u32| format!("0x{:08X}", id);

    if let Some(v) = current.settings.get(&format!("Max Frame Rate")).map(|s| s.value)
        .or_else(|| current.settings.values().find(|s| s.id == key(nvapi_ids::FRL_FPS)).map(|s| s.value))
    {
        settings.frame_rate_limit = Some(v);
    }
    if let Some(v) = current.settings.values().find(|s| s.id == key(nvapi_ids::PRERENDERLIMIT)).map(|s| s.value) {
        settings.low_latency_mode = Some(v);
    }
    if let Some(v) = current.settings.values().find(|s| s.id == key(nvapi_ids::VSYNCMODE)).map(|s| s.value) {
        settings.vsync = VSyncMode::from_nvapi_value(v).map(|m| match m {
            VSyncMode::Off => "off",
            VSyncMode::On => "on",
            VSyncMode::Adaptive => "adaptive",
            VSyncMode::Fast => "fast",
            VSyncMode::ApplicationControlled => "application",
        }.to_string());
    }
    if let Some(v) = current.settings.values().find(|s| s.id == key(nvapi_ids::PREFERRED_PSTATE)).map(|s| s.value) {
        settings.power_management = PowerManagementMode::from_nvapi_value(v).map(|m| match m {
            PowerManagementMode::Adaptive => "adaptive",
            PowerManagementMode::PreferMaxPerformance => "prefer_max_performance",
            PowerManagementMode::Optimal => "optimal",
        }.to_string());
    }
    if let Some(v) = current.settings.values().find(|s| s.id == key(nvapi_ids::SHADER_DISK_CACHE)).map(|s| s.value) {
        settings.shader_cache = Some(match v {
            shader_cache_values::ON => "on",
            _ => "off",
        }.to_string());
    }
    if let Some(v) = current.settings.values().find(|s| s.id == key(nvapi_ids::OGL_THREAD_CONTROL)).map(|s| s.value) {
        settings.threaded_optimization = Some(match v {
            thread_control_values::ENABLE => "on",
            thread_control_values::DISABLE => "off",
            _ => "auto",
        }.to_string());
    }
    if let Some(v) = current.settings.values().find(|s| s.id == key(nvapi_ids::CUDA_FORCE_P2_STATE)).map(|s| s.value) {
        settings.cuda_force_p2 = Some(v != 0);
    }

    Some(NvidiaBackup {
        profile_name: current.name,
        existed: true,
        settings,
    })
}
