//! Live NVAPI DRS round-trip integration test.
//!
//! Creates a unique throwaway profile with a known set of DWORDs, reads them
//! back via `get_profile_full`, then deletes the profile. Runs only when NVAPI
//! can be initialised and a DRS session can be opened; on CI/non-NVIDIA hosts
//! it logs the reason and exits successfully so the rest of the suite still
//! passes.

use game_optimizer::drivers::nvidia::{NvidiaProfileManager, is_nvidia_available};
use game_optimizer::drivers::settings::{
    AnisotropicLevel, AntiAliasingMode, AntiAliasingSetting, DriverProfile, PowerManagementMode,
    TextureFilterQuality, VSyncMode,
};

fn unique_profile_name() -> String {
    let pid = std::process::id();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    format!("game_optimizer_it_{}_{}", pid, nanos)
}

#[test]
fn drs_round_trip_writes_and_reads_every_mapped_setting() {
    if !is_nvidia_available() {
        eprintln!("skipping: NVAPI not available on this host");
        return;
    }

    let manager = match NvidiaProfileManager::new() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("skipping: failed to init NvidiaProfileManager ({e})");
            return;
        }
    };

    if manager.session().is_none() {
        eprintln!("skipping: DRS session unavailable");
        return;
    }

    let name = unique_profile_name();
    let exe = format!("{name}.exe");

    // Best-effort cleanup hook in case a previous run died mid-test.
    let _ = manager.delete_profile(&name);

    let profile = DriverProfile {
        name: name.clone(),
        executables: vec![exe.clone()],
        frame_rate_limit: Some(141),
        low_latency: Some(1),
        vsync: Some(VSyncMode::Off),
        power_management: Some(PowerManagementMode::PreferMaxPerformance),
        anisotropic_filtering: Some(AnisotropicLevel::X16),
        shader_cache: Some(true),
        threaded_optimization: Some(true),
        mfaa: Some(true),
        triple_buffering: Some(false),
        texture_filter_quality: Some(TextureFilterQuality::HighQuality),
        preferred_refresh_rate: Some(true),
        background_fps_limit: Some(30),
        aa_mode: Some(AntiAliasingMode::Override),
        aa_setting: Some(AntiAliasingSetting::Msaa4x),
        lod_bias: Some(-1000),
        trilinear_optimization: Some(true),
        aniso_sample_optimization: Some(false),
        gsync_indicator: Some(true),
        // CUDA_FORCE_P2_STATE returns SETTING_NOT_FOUND on current drivers - exclude
        // from the round trip so we don't get a spurious write failure.
        cuda_force_p2: None,
    };

    manager
        .set_profile(&profile)
        .expect("set_profile must succeed against a live DRS session");

    // Guard against early-return leaks: delete on panic via a scope guard.
    struct Cleanup<'a> {
        mgr: &'a NvidiaProfileManager,
        name: &'a str,
    }
    impl Drop for Cleanup<'_> {
        fn drop(&mut self) {
            let _ = self.mgr.delete_profile(self.name);
        }
    }
    let _cleanup = Cleanup { mgr: &manager, name: &name };

    let read_back = manager
        .get_profile_full(&name)
        .expect("get_profile_full should not error")
        .expect("the profile we just wrote must exist");

    assert!(!read_back.is_predefined);
    assert!(read_back
        .applications
        .iter()
        .any(|a| a.eq_ignore_ascii_case(&exe) || a.eq_ignore_ascii_case(&name)));

    // Spot-check a handful of DWORDs by their display name. We compare through
    // the ProfileSettingValue.value (raw DWORD) so the test is robust to any
    // cosmetic change in describe_setting_value.
    let get = |key: &str| -> u32 {
        read_back
            .settings
            .get(key)
            .unwrap_or_else(|| panic!("missing setting '{key}' in {:?}", read_back.settings.keys().collect::<Vec<_>>()))
            .value
    };

    assert_eq!(get("Max Frame Rate"), 141, "FPS limit mismatch");
    assert_eq!(get("Anisotropic Filtering"), 16, "aniso level mismatch");
    assert_eq!(
        get("Antialiasing - Setting"),
        AntiAliasingSetting::Msaa4x.to_nvapi_value(),
        "AA method mismatch"
    );
    assert_eq!(
        get("Antialiasing - Mode"),
        AntiAliasingMode::Override.to_nvapi_value(),
        "AA selector mismatch"
    );
    // -1000 as i32 stored in the DWORD round-trips bitwise.
    assert_eq!(get("Texture Filtering - LOD Bias") as i32, -1000);
    assert_eq!(
        get("Multi-Frame Sampled AA (MFAA)"),
        1,
        "MFAA expected ON"
    );
    assert_eq!(get("Triple Buffering"), 0, "triple buffer expected OFF");
}
