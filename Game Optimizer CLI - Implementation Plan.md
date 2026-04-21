# Game Optimizer CLI
A Rust CLI application that uses AI and online data sources to recommend optimal game settings for NVIDIA driver profiles, in-game settings, Lossless Scaling, and RTSS framerate limiting.
## Problem Statement
Optimizing game settings is tedious and requires cross-referencing multiple sources (benchmarks, forums, PCGamingWiki) while understanding hardware capabilities. This tool automates that process by aggregating data and using AI to generate personalized recommendations.
## Current State
**Existing Rust ecosystem:**
* `nvapi-rs` (nvapi-hi, nvapi, nvapi-sys) - NVIDIA driver API bindings
* `rtss-sys` - RivaTuner Statistics Server SDK bindings
* `reqwest` - HTTP client for API calls
* `scraper` - HTML parsing for web scraping
* `clap` - CLI argument parsing
**Data sources:**
* PCGamingWiki Cargo API (game metadata, settings, config paths)
* Steam API (game detection, AppID lookup)
* Digital Foundry / Hardware Unboxed (benchmark data via scraping)
* Claude API (AI recommendations)
**Target integrations:**
* NVIDIA Control Panel profiles (via NVAPI)
* RTSS framerate limiting profiles
* Lossless Scaling config.ini and game profiles
* DynamicFPSLimiter (future)
## Proposed Architecture
```warp-runnable-command
game-optimizer/
├── Cargo.toml
├── src/
│   ├── main.rs                 # CLI entry point
│   ├── lib.rs                  # Library exports
│   ├── config.rs               # App configuration (API keys, paths)
│   ├── hardware/
│   │   ├── mod.rs
│   │   ├── gpu.rs              # GPU detection via NVAPI
│   │   └── system.rs           # CPU, RAM, monitor info
│   ├── drivers/
│   │   ├── mod.rs
│   │   ├── nvidia.rs           # NVAPI profile management
│   │   └── settings.rs         # Driver setting definitions
│   ├── tools/
│   │   ├── mod.rs
│   │   ├── rtss.rs             # RTSS integration
│   │   ├── lossless.rs         # Lossless Scaling profiles
│   │   └── dynamic_fps.rs      # DynamicFPSLimiter (future)
│   ├── data/
│   │   ├── mod.rs
│   │   ├── pcgamingwiki.rs     # PCGamingWiki API client
│   │   ├── steam.rs            # Steam game detection
│   │   ├── benchmarks.rs       # Benchmark data aggregation
│   │   └── cache.rs            # Local caching layer
│   ├── ai/
│   │   ├── mod.rs
│   │   ├── claude.rs           # Claude API client
│   │   └── prompts.rs          # Prompt templates
│   └── commands/
│       ├── mod.rs
│       ├── detect.rs           # Detect running/installed games
│       ├── analyze.rs          # Analyze game + hardware
│       ├── recommend.rs        # Generate recommendations
│       └── apply.rs            # Apply settings
└── tests/
```
## Phase 1: Foundation & Driver Settings
**Goal:** CLI that can read/write NVIDIA driver profiles for games.
**Tasks:**
1. Project scaffolding with Cargo workspace
2. Hardware detection module (GPU model, VRAM, driver version via nvapi-hi)
3. NVAPI profile management:
    * List existing profiles
    * Read profile settings
    * Create/modify game-specific profiles
    * Key settings: Max Frame Rate, Power Management Mode, Threaded Optimization, Low Latency Mode, Shader Cache, Texture Filtering, etc.
4. CLI commands: `detect-gpu`, `list-profiles`, `get-profile <game>`, `set-profile <game> <setting> <value>`
**Key NVAPI settings to expose:**
* `PRERENDERLIMIT` (Max pre-rendered frames / Low Latency)
* `PS_FRAMERATE_LIMITER` (Frame Rate Limit)
* `VSYNCMODE` (Vertical Sync)
* `PS_TEXFILTER_ANISO_OPTS2` (Anisotropic Filtering)
* `PS_SHADERDISKCACHE` (Shader Cache)
* `OGL_THREAD_CONTROL` (Threaded Optimization)
* `PREFERRED_PSTATE` (Power Management Mode)
## Phase 2: Data Sources Integration
**Goal:** Aggregate game optimization data from multiple sources.
**Tasks:**
1. PCGamingWiki client:
    * Query by Steam AppID or game name
    * Fetch: supported APIs (DX11/12/Vulkan), display settings, known issues, config file paths
    * Parse Cargo API responses
2. Steam integration:
    * Detect installed games from Steam library
    * Get AppIDs for game identification
    * Read game launch options
3. Benchmark data aggregation:
    * Scrape/parse Digital Foundry optimized settings articles
    * Hardware Unboxed settings recommendations
    * Cache results locally with TTL
4. Local cache layer (SQLite or JSON files in `~/.local/share/game-optimizer/`)
## Phase 3: AI Recommendations
**Goal:** Use Claude to analyze hardware + game data and generate optimal settings.
**Tasks:**
1. Claude API client with streaming support
2. Prompt engineering for settings recommendations:
    * Input: GPU model, VRAM, target resolution, target FPS, game name, PCGamingWiki data, benchmark references
    * Output: Structured JSON with driver settings, in-game settings, reasoning
3. Recommendation commands:
    * `recommend <game>` - Full analysis and recommendations
    * `recommend <game> --driver-only` - Just NVAPI settings
    * `recommend <game> --quick` - Skip web lookups, use cache/heuristics
4. Interactive mode for follow-up questions
**Example prompt structure:**
```warp-runnable-command
You are a PC gaming optimization expert. Given:
- GPU: RTX 5080 (16GB VRAM)
- Target: 1440p @ 144Hz with G-Sync
- Game: {game_name}
- PCGamingWiki data: {pcgw_data}
- Known benchmarks: {benchmark_data}
Recommend optimal settings for:
1. NVIDIA Control Panel profile
2. In-game graphics settings
3. Lossless Scaling configuration (if beneficial)
4. RTSS framerate limit
Provide JSON output with reasoning.
```
## Phase 4: RTSS & Lossless Scaling Integration
**Goal:** Control framerate limiting and frame generation tools.
**Tasks:**
1. RTSS integration via rtss-sys:
    * Read/write application profiles
    * Set framerate limits per game
    * Query current framerate/frametime data
2. Lossless Scaling profile management:
    * Locate config.ini (`%LOCALAPPDATA%\LosslessScaling\` or Steam userdata)
    * Parse and modify game profiles
    * Settings: LSFG version, Mode (Fixed/Adaptive), Flow Scale, Queue Target, Scaling Type
3. Coordinated settings:
    * RTSS limit + LS multiplier calculations (e.g., 48 FPS cap × 3x = 144Hz output)
    * Warn about incompatible combinations
4. CLI commands: `rtss list`, `rtss set <game> --fps <limit>`, `ls profile <game>`, `ls set <game> --mode adaptive`
## Phase 5: Full Workflow & Polish
**Goal:** End-to-end optimization workflow.
**Tasks:**
1. `optimize <game>` command:
    * Detect game
    * Gather hardware info
    * Fetch online data
    * Generate AI recommendations
    * Preview changes
    * Apply with confirmation
2. Profile export/import (JSON/TOML)
3. Backup/restore for all modified settings
4. Dry-run mode for all apply operations
5. Logging and diagnostics
## Phase 6: GUI (Future)
**Goal:** Desktop GUI for non-CLI users.
**Technology:** Tauri (Rust backend + web frontend)
**Scope:** Reuse all library code from CLI, add web UI layer.
## Dependencies
```toml
[dependencies]
nvapi-hi = "0.1"           # NVIDIA API high-level bindings
rtss-sys = "0.1"           # RTSS SDK bindings
reqwest = { version = "0.12", features = ["json"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
clap = { version = "4", features = ["derive"] }
scraper = "0.20"           # HTML parsing
tracing = "0.1"            # Logging
tracing-subscriber = "0.3"
dirs = "5"                 # XDG directories
toml = "0.8"               # Config files
rusqlite = "0.32"          # Local cache DB
```
## Risks & Mitigations
* **nvapi-rs maintenance:** Crate is 3+ years old. May need to fork/update for RTX 50 series support. Fallback: FFI bindings directly to NVAPI SDK.
* **Lossless Scaling config format:** No official API. Config format may change between versions. Mitigation: Version detection and format validation.
* **Web scraping fragility:** Benchmark sites may change layouts. Mitigation: Multiple sources, graceful degradation, manual override.
* **AI hallucination:** Claude may suggest invalid settings. Mitigation: Validate all recommendations against known valid values before applying.
## Success Criteria
1. Can detect GPU and list current NVAPI profiles
2. Can query PCGamingWiki for any Steam game
3. Can generate valid recommendations via Claude API
4. Can apply NVAPI profile changes
5. Can set RTSS framerate limits
6. End-to-end `optimize <game>` workflow functional
