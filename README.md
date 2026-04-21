# game-optimizer

AI-assisted CLI that aggregates data from PCGamingWiki, benchmark sources, and Claude to recommend and **apply** optimal game settings through the NVIDIA Control Panel (via NVAPI DRS), RivaTuner Statistics Server (RTSS), and Lossless Scaling.

- Platform: Windows (x86_64)
- GPU: NVIDIA (NVAPI — tested on RTX 50-series, driver 595+)
- Language: Rust 2021

## Quickstart

```powershell
cargo build --release
.\target\release\game_optimizer.exe detect-gpu
.\target\release\game_optimizer.exe list-games
.\target\release\game_optimizer.exe optimize "Elden Ring" --quick --dry-run --fps 144
```

Full run (creates a backup first, writes to the NVIDIA driver, confirms):

```powershell
game_optimizer optimize "Elden Ring" --quick --fps 144 -y
```

Restore if needed:

```powershell
game_optimizer backup list
game_optimizer backup restore <backup-id> -y
```

## Commands

| Command | What it does |
|---|---|
| `detect-gpu` / `list-gpus` | GPU info via NVAPI |
| `list-games` | Steam / Epic / GOG / EA / Xbox game discovery |
| `game-info <query>` | PCGamingWiki lookup (engine, HDR/RT support, etc.) |
| `benchmarks <game>` | Scrape Digital Foundry / Hardware Unboxed references |
| `list-profiles` | NVIDIA driver profiles (installed-games filter by default) |
| `get-profile <name>` | Full read of a profile's DRS state |
| `set-profile <name> …` | Typed flags for every mapped DRS setting (see below) |
| `delete-profile <name>` | Delete a user profile (predefined profiles refused) |
| `remove-application <profile> <exe>` | Detach one exe from a profile |
| `recommend <game> [--quick]` | AI (or heuristic) recommendation, read-only |
| `optimize <game>` | Full workflow: gather → recommend → preview → backup → write → confirm |
| `backup {list,show,restore,delete,export,import}` | Backup / profile portability |
| `rtss {status,list,get,set}` | RTSS FPS limiter management |
| `ls {status,list,get,set,calc}` | Lossless Scaling LSFG profile management |
| `config {show,set,init,path}` | API keys, target resolution/refresh-rate, G-Sync |

Global flags: `--json`, `-v/--verbose`, `--debug`.

## `set-profile` flag coverage

Every typed field on `DriverProfile` is exposed:

```
--fps-limit <u32>                   FRL_FPS
--low-latency <0|1|2>               PRERENDERLIMIT
--vsync <off|on|adaptive|fast|app>  VSYNCMODE (+ VSYNC_TEAR_CONTROL for Adaptive)
--power <adaptive|max-performance|optimal>  PREFERRED_PSTATE
--anisotropic <0|2|4|8|16>          ANISO_MODE_SELECTOR + LEVEL
--mfaa <bool>                       MAXWELL_B_SAMPLE_INTERLEAVE
--triple-buffer <bool>              OGL_TRIPLE_BUFFER
--texture-quality <high_quality|quality|performance|high_performance>
--preferred-refresh-rate <bool>     REFRESH_RATE_OVERRIDE
--background-fps <u32>              APPIDLE_DYNAMIC_FRL_FPS
--aa-mode <app|override|enhance>    AA_MODE_SELECTOR
--aa-setting <none|2x|4x|8x>        AA_MODE_METHOD
--lod-bias=<i32>                    LODBIASADJUST (use `=` for negative values)
--trilinear-optimization <bool>     PS_TEXFILTER_DISABLE_TRILIN_SLOPE
--aniso-sample-optimization <bool>  PS_TEXFILTER_ANISO_OPTS2
--gsync-indicator <bool>            VRR_OVERLAY_INDICATOR
--cuda-force-p2 <bool>              CUDA_FORCE_P2_STATE*
```

\* `CUDA_FORCE_P2_STATE` is not in the current public NVAPI header; driver 595+ returns `NVAPI_SETTING_NOT_FOUND (-137)`.

## Architecture

```
src/
├── main.rs                 CLI (clap subcommands)
├── lib.rs                  Public re-exports
├── config.rs               XDG config (~/.config/game-optimizer/)
├── backup.rs               SettingsBackup, BackupManager, GameProfile export/import
├── hardware/               GPU detection (NVAPI), system info
├── drivers/
│   ├── drs.rs              NVAPI DRS FFI (size-asserted NVDRS_SETTING_V1, NVDRS_APPLICATION_V4)
│   ├── settings.rs         Setting ID map + value encoders + DriverProfile
│   └── nvidia.rs           NvidiaProfileManager: find/create/write/read/delete/reset
├── data/                   Steam/Epic/GOG/EA/Xbox detection, PCGamingWiki, benchmarks, cache
├── ai/                     Claude client + prompt templates + DriverSettings JSON schema
├── tools/                  RTSS integration, Lossless Scaling (quick-xml)
└── commands/               `detect-gpu`, `list-gpus` handlers
```

## DRS write support

Struct layouts in `src/drivers/drs.rs` are pinned to NVIDIA's [published `NvApiDriverSettings.h`](https://github.com/NVIDIA/nvapi) at compile time:

```rust
const _: () = {
    assert!(size_of::<NVDRS_BINARY_SETTING>() == 4100);
    assert!(size_of::<NVDRS_SETTING_VALUE>()  == 4100);
    assert!(size_of::<NVDRS_APPLICATION>()    == 20492);  // V4
    assert!(size_of::<NVDRS_SETTING>()        == 12320);  // V1
};
```

If NVIDIA ships a newer struct revision, `cargo check` fails with a clear message instead of the driver silently rejecting writes with `NVAPI_INCOMPATIBLE_STRUCT_VERSION (-9)`.

The FFI layer wraps:

- `DRS_CreateSession` / `LoadSettings` / `SaveSettings` / `DestroySession`
- `DRS_CreateProfile` / `FindProfileByName` / `EnumProfiles` / `GetProfileInfo` / `DeleteProfile`
- `DRS_CreateApplication` / `EnumApplications` / `DeleteApplication`
- `DRS_GetSetting` / `SetSetting` / `DeleteProfileSetting`

All setting IDs used by `NvidiaSetting::setting_id()` come from the public NVAPI header. `CUDA_FORCE_P2_STATE` is the lone community-sourced exception and documented as such.

## Safety notes

- Writing to the NVIDIA driver affects every game. `optimize` always creates a `SettingsBackup` first (unless `--no-backup`); `backup restore` re-applies the captured DWORDs via the same code path.
- Writes require a successful `CreateSession` + `LoadSettings`; neither the tool nor the driver require administrator on standard configurations.
- Predefined (NVIDIA-shipped) profiles cannot be deleted; `delete-profile` refuses them.
- AI recommendations are validated against a fixed JSON schema (see `src/ai/prompts.rs`). Unknown string values for enum fields are dropped rather than written.

## Testing

```powershell
cargo test --lib          # 18+ unit tests (value encoders, setting-ID coverage, header sanity)
cargo clippy --no-deps    # zero errors
```

## Building

- Toolchain: stable Rust
- Target: `x86_64-pc-windows-msvc`
- No special build flags required. NVAPI is loaded at runtime via `nvapi-sys`.

## License

MIT.
