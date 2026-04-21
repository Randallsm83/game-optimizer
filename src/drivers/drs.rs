//! NVIDIA DRS (Driver Settings) API bindings
//!
//! Low-level FFI bindings to NVAPI DRS functions for profile management.
//!
//! # Safety contract
//!
//! `NvDRSProfileHandle` is `*mut c_void` because NVAPI returns opaque pointers,
//! but callers never construct these themselves - they only receive them via
//! [`DrsSession::enumerate_profiles`], [`DrsSession::find_profile`], or
//! [`DrsSession::create_profile`] on an active session. The session's own
//! lifetime governs their validity. The lint
//! `clippy::not_unsafe_ptr_arg_deref` is therefore suppressed at the module
//! level: methods that dereference these handles are safe from the caller's
//! perspective as long as the handle came from the same `DrsSession`.

#![allow(non_camel_case_types)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::c_void;
use std::mem::{size_of, zeroed};
use std::ptr;

use nvapi_sys::nvapi::nvapi_QueryInterface;
use nvapi_sys::nvid::Api;

/// DRS session handle
pub type NvDRSSessionHandle = *mut c_void;

/// DRS profile handle  
pub type NvDRSProfileHandle = *mut c_void;

/// NVAPI status codes
pub type NvAPI_Status = i32;

pub const NVAPI_OK: NvAPI_Status = 0;
pub const NVAPI_ERROR: NvAPI_Status = -1;
pub const NVAPI_LIBRARY_NOT_FOUND: NvAPI_Status = -2;
pub const NVAPI_NO_IMPLEMENTATION: NvAPI_Status = -3;
pub const NVAPI_API_NOT_INITIALIZED: NvAPI_Status = -4;
pub const NVAPI_INVALID_ARGUMENT: NvAPI_Status = -5;
pub const NVAPI_NVIDIA_DEVICE_NOT_FOUND: NvAPI_Status = -6;
pub const NVAPI_END_ENUMERATION: NvAPI_Status = -7;
pub const NVAPI_INVALID_HANDLE: NvAPI_Status = -8;
pub const NVAPI_INCOMPATIBLE_STRUCT_VERSION: NvAPI_Status = -9;
pub const NVAPI_HANDLE_INVALIDATED: NvAPI_Status = -10;
pub const NVAPI_OPENGL_CONTEXT_NOT_CURRENT: NvAPI_Status = -11;
pub const NVAPI_INVALID_POINTER: NvAPI_Status = -14;
pub const NVAPI_NO_GL_EXPERT: NvAPI_Status = -12;
pub const NVAPI_INSTRUMENTATION_DISABLED: NvAPI_Status = -13;
pub const NVAPI_ACCESS_DENIED: NvAPI_Status = -15;
/// Returned by `SetSetting` when the setting ID is not recognised by the driver.
pub const NVAPI_SETTING_NOT_FOUND: NvAPI_Status = -137;

/// Translate a common NVAPI status code to a short human-readable name.
/// Covers the codes we care about; unknown values fall back to `"UNKNOWN"`.
pub fn status_message(status: NvAPI_Status) -> &'static str {
    match status {
        NVAPI_OK => "OK",
        NVAPI_ERROR => "ERROR",
        NVAPI_LIBRARY_NOT_FOUND => "LIBRARY_NOT_FOUND",
        NVAPI_NO_IMPLEMENTATION => "NO_IMPLEMENTATION",
        NVAPI_API_NOT_INITIALIZED => "API_NOT_INITIALIZED",
        NVAPI_INVALID_ARGUMENT => "INVALID_ARGUMENT",
        NVAPI_NVIDIA_DEVICE_NOT_FOUND => "NVIDIA_DEVICE_NOT_FOUND",
        NVAPI_END_ENUMERATION => "END_ENUMERATION",
        NVAPI_INVALID_HANDLE => "INVALID_HANDLE",
        NVAPI_INCOMPATIBLE_STRUCT_VERSION => "INCOMPATIBLE_STRUCT_VERSION",
        NVAPI_HANDLE_INVALIDATED => "HANDLE_INVALIDATED",
        NVAPI_OPENGL_CONTEXT_NOT_CURRENT => "OPENGL_CONTEXT_NOT_CURRENT",
        NVAPI_NO_GL_EXPERT => "NO_GL_EXPERT",
        NVAPI_INSTRUMENTATION_DISABLED => "INSTRUMENTATION_DISABLED",
        NVAPI_INVALID_POINTER => "INVALID_POINTER",
        NVAPI_ACCESS_DENIED => "ACCESS_DENIED",
        NVAPI_SETTING_NOT_FOUND => "SETTING_NOT_FOUND",
        _ => "UNKNOWN",
    }
}

/// Maximum profile name length
pub const NVAPI_UNICODE_STRING_MAX: usize = 2048;
/// Maximum buffer size for `NVDRS_BINARY_SETTING::value_data` (from public NVAPI header).
pub const NVAPI_BINARY_DATA_MAX: usize = 4096;
/// Retained for array-of-values structs. Not used by `NVDRS_SETTING_VALUE`.
pub const NVAPI_SETTING_MAX_VALUES: usize = 100;

/// Short string buffer
pub type NvAPI_ShortString = [u8; 64];
pub type NvAPI_UnicodeString = [u16; NVAPI_UNICODE_STRING_MAX];

/// Profile info structure version
pub const NVDRS_PROFILE_VER: u32 = 0x00010000 | size_of::<NVDRS_PROFILE>() as u32;

/// Application info structure version (matches `NVDRS_APPLICATION_VER_V4`).
pub const NVDRS_APPLICATION_VER: u32 = 0x00040000 | size_of::<NVDRS_APPLICATION>() as u32;

/// Setting structure version (matches `NVDRS_SETTING_VER1`).
pub const NVDRS_SETTING_VER: u32 = 0x00010000 | size_of::<NVDRS_SETTING>() as u32;

/// Profile information
#[repr(C)]
#[derive(Clone)]
pub struct NVDRS_PROFILE {
    pub version: u32,
    pub profile_name: NvAPI_UnicodeString,
    pub gpu_support: u32,
    pub is_predefined: u32,
    pub num_of_apps: u32,
    pub num_of_settings: u32,
}

impl Default for NVDRS_PROFILE {
    fn default() -> Self {
        unsafe {
            let mut profile: Self = zeroed();
            profile.version = NVDRS_PROFILE_VER;
            profile
        }
    }
}

/// Application information, matching `NVDRS_APPLICATION_V4`.
///
/// `flags` packs three NVAPI bit-fields into a single DWORD:
/// - bit 0: `isMetro`
/// - bit 1: `isCommandLine`
/// - bits 2..31: reserved (must be 0)
#[repr(C)]
#[derive(Clone)]
pub struct NVDRS_APPLICATION {
    pub version: u32,
    pub is_predefined: u32,
    pub app_name: NvAPI_UnicodeString,
    pub user_friendly_name: NvAPI_UnicodeString,
    pub launcher: NvAPI_UnicodeString,
    pub file_in_folder: NvAPI_UnicodeString,
    pub flags: u32,
    pub command_line: NvAPI_UnicodeString,
}

impl Default for NVDRS_APPLICATION {
    fn default() -> Self {
        unsafe {
            let mut app: Self = zeroed();
            app.version = NVDRS_APPLICATION_VER;
            app
        }
    }
}

/// Setting type
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NVDRS_SETTING_TYPE {
    NVDRS_DWORD_TYPE = 0,
    NVDRS_BINARY_TYPE = 1,
    NVDRS_STRING_TYPE = 2,
    NVDRS_WSTRING_TYPE = 3,
}

/// Binary setting payload used inside `NVDRS_SETTING_VALUE` for binary-typed settings.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct NVDRS_BINARY_SETTING {
    pub value_length: u32,
    pub value_data: [u8; NVAPI_BINARY_DATA_MAX],
}

/// Setting value union. Layout matches NVIDIA's public `NVDRS_SETTING_V1`:
/// a 32-bit DWORD, an `NVDRS_BINARY_SETTING`, or a wide string.
/// The union size is driven by `NVDRS_BINARY_SETTING` (4100 bytes).
#[repr(C)]
#[derive(Clone, Copy)]
pub union NVDRS_SETTING_VALUE {
    pub u32_value: u32,
    pub binary_value: NVDRS_BINARY_SETTING,
    pub wstring_value: NvAPI_UnicodeString,
}

/// Setting information
#[repr(C)]
pub struct NVDRS_SETTING {
    pub version: u32,
    pub setting_name: NvAPI_UnicodeString,
    pub setting_id: u32,
    pub setting_type: u32,
    pub setting_location: u32,
    pub is_current_predefined: u32,
    pub is_predefined_valid: u32,
    pub predefined_value: NVDRS_SETTING_VALUE,
    pub current_value: NVDRS_SETTING_VALUE,
}

impl Default for NVDRS_SETTING {
    fn default() -> Self {
        unsafe {
            let mut setting: Self = zeroed();
            setting.version = NVDRS_SETTING_VER;
            setting
        }
    }
}

// ---------------------------------------------------------------------------
// Compile-time struct-size guards.
//
// The NVAPI driver validates struct sizes (encoded in the `version` field) and
// rejects any mismatch with `NVAPI_INCOMPATIBLE_STRUCT_VERSION`. These asserts
// pin our Rust layouts to the sizes the driver expects:
//
// - NVDRS_BINARY_SETTING : u32 (4) + [u8; 4096] = 4100
// - NVDRS_SETTING_VALUE  : driven by NVDRS_BINARY_SETTING = 4100
// - NVDRS_APPLICATION_V4 : u32*2 + [u16;2048]*4 + u32 + [u16;2048] = 20492
// - NVDRS_SETTING_V1     : u32 + [u16;2048] + u32*5 + 2 * NVDRS_SETTING_VALUE = 12320
//
// If any of these fire, the FFI struct has drifted from the public NVAPI header
// and runtime `SetSetting`/`CreateApplication` calls will return status -9.
const _: () = {
    assert!(
        size_of::<NVDRS_BINARY_SETTING>() == 4100,
        "NVDRS_BINARY_SETTING size drift - expected 4100 bytes",
    );
    assert!(
        size_of::<NVDRS_SETTING_VALUE>() == 4100,
        "NVDRS_SETTING_VALUE size drift - expected 4100 bytes",
    );
    assert!(
        size_of::<NVDRS_APPLICATION>() == 20492,
        "NVDRS_APPLICATION (V4) size drift - expected 20492 bytes",
    );
    assert!(
        size_of::<NVDRS_SETTING>() == 12320,
        "NVDRS_SETTING (V1) size drift - expected 12320 bytes",
    );
};

/// Function pointer types
type NvAPI_DRS_CreateSession_t = unsafe extern "C" fn(*mut NvDRSSessionHandle) -> NvAPI_Status;
type NvAPI_DRS_DestroySession_t = unsafe extern "C" fn(NvDRSSessionHandle) -> NvAPI_Status;
type NvAPI_DRS_LoadSettings_t = unsafe extern "C" fn(NvDRSSessionHandle) -> NvAPI_Status;
type NvAPI_DRS_SaveSettings_t = unsafe extern "C" fn(NvDRSSessionHandle) -> NvAPI_Status;
type NvAPI_DRS_GetNumProfiles_t = unsafe extern "C" fn(NvDRSSessionHandle, *mut u32) -> NvAPI_Status;
type NvAPI_DRS_EnumProfiles_t = unsafe extern "C" fn(NvDRSSessionHandle, u32, *mut NvDRSProfileHandle) -> NvAPI_Status;
type NvAPI_DRS_GetProfileInfo_t = unsafe extern "C" fn(NvDRSSessionHandle, NvDRSProfileHandle, *mut NVDRS_PROFILE) -> NvAPI_Status;
type NvAPI_DRS_EnumApplications_t = unsafe extern "C" fn(NvDRSSessionHandle, NvDRSProfileHandle, u32, *mut u32, *mut NVDRS_APPLICATION) -> NvAPI_Status;
type NvAPI_DRS_GetBaseProfile_t = unsafe extern "C" fn(NvDRSSessionHandle, *mut NvDRSProfileHandle) -> NvAPI_Status;
type NvAPI_DRS_FindProfileByName_t = unsafe extern "C" fn(NvDRSSessionHandle, *const u16, *mut NvDRSProfileHandle) -> NvAPI_Status;
type NvAPI_DRS_GetSetting_t = unsafe extern "C" fn(NvDRSSessionHandle, NvDRSProfileHandle, u32, *mut NVDRS_SETTING) -> NvAPI_Status;
type NvAPI_DRS_SetSetting_t = unsafe extern "C" fn(NvDRSSessionHandle, NvDRSProfileHandle, *const NVDRS_SETTING) -> NvAPI_Status;
type NvAPI_DRS_CreateProfile_t = unsafe extern "C" fn(NvDRSSessionHandle, *mut NVDRS_PROFILE, *mut NvDRSProfileHandle) -> NvAPI_Status;
type NvAPI_DRS_CreateApplication_t = unsafe extern "C" fn(NvDRSSessionHandle, NvDRSProfileHandle, *mut NVDRS_APPLICATION) -> NvAPI_Status;
type NvAPI_DRS_DeleteProfile_t = unsafe extern "C" fn(NvDRSSessionHandle, NvDRSProfileHandle) -> NvAPI_Status;
type NvAPI_DRS_DeleteApplication_t = unsafe extern "C" fn(NvDRSSessionHandle, NvDRSProfileHandle, *const u16) -> NvAPI_Status;
type NvAPI_DRS_DeleteProfileSetting_t = unsafe extern "C" fn(NvDRSSessionHandle, NvDRSProfileHandle, u32) -> NvAPI_Status;

/// DRS API function pointers
pub struct DrsApi {
    create_session: NvAPI_DRS_CreateSession_t,
    destroy_session: NvAPI_DRS_DestroySession_t,
    load_settings: NvAPI_DRS_LoadSettings_t,
    save_settings: NvAPI_DRS_SaveSettings_t,
    get_num_profiles: NvAPI_DRS_GetNumProfiles_t,
    enum_profiles: NvAPI_DRS_EnumProfiles_t,
    get_profile_info: NvAPI_DRS_GetProfileInfo_t,
    enum_applications: NvAPI_DRS_EnumApplications_t,
    get_base_profile: NvAPI_DRS_GetBaseProfile_t,
    find_profile_by_name: NvAPI_DRS_FindProfileByName_t,
    get_setting: NvAPI_DRS_GetSetting_t,
    set_setting: NvAPI_DRS_SetSetting_t,
    create_profile: NvAPI_DRS_CreateProfile_t,
    create_application: NvAPI_DRS_CreateApplication_t,
    delete_profile: NvAPI_DRS_DeleteProfile_t,
    delete_application: NvAPI_DRS_DeleteApplication_t,
    delete_profile_setting: NvAPI_DRS_DeleteProfileSetting_t,
}

#[allow(non_camel_case_types)]
impl DrsApi {
    /// Load DRS API function pointers
    pub fn load() -> Option<Self> {
        unsafe {
            let get_fn = |api: Api| -> Option<*const c_void> {
                match nvapi_QueryInterface(api.id()) {
                    Ok(ptr) if ptr != 0 => Some(ptr as *const c_void),
                    _ => None,
                }
            };

            Some(Self {
                create_session: std::mem::transmute(get_fn(Api::NvAPI_DRS_CreateSession)?),
                destroy_session: std::mem::transmute(get_fn(Api::NvAPI_DRS_DestroySession)?),
                load_settings: std::mem::transmute(get_fn(Api::NvAPI_DRS_LoadSettings)?),
                save_settings: std::mem::transmute(get_fn(Api::NvAPI_DRS_SaveSettings)?),
                get_num_profiles: std::mem::transmute(get_fn(Api::NvAPI_DRS_GetNumProfiles)?),
                enum_profiles: std::mem::transmute(get_fn(Api::NvAPI_DRS_EnumProfiles)?),
                get_profile_info: std::mem::transmute(get_fn(Api::NvAPI_DRS_GetProfileInfo)?),
                enum_applications: std::mem::transmute(get_fn(Api::NvAPI_DRS_EnumApplications)?),
                get_base_profile: std::mem::transmute(get_fn(Api::NvAPI_DRS_GetBaseProfile)?),
                find_profile_by_name: std::mem::transmute(get_fn(Api::NvAPI_DRS_FindProfileByName)?),
                get_setting: std::mem::transmute(get_fn(Api::NvAPI_DRS_GetSetting)?),
                set_setting: std::mem::transmute(get_fn(Api::NvAPI_DRS_SetSetting)?),
                create_profile: std::mem::transmute(get_fn(Api::NvAPI_DRS_CreateProfile)?),
                create_application: std::mem::transmute(get_fn(Api::NvAPI_DRS_CreateApplication)?),
                delete_profile: std::mem::transmute(get_fn(Api::NvAPI_DRS_DeleteProfile)?),
                delete_application: std::mem::transmute(get_fn(Api::NvAPI_DRS_DeleteApplication)?),
                delete_profile_setting: std::mem::transmute(get_fn(Api::NvAPI_DRS_DeleteProfileSetting)?),
            })
        }
    }

    pub unsafe fn create_session(&self, handle: *mut NvDRSSessionHandle) -> NvAPI_Status {
        (self.create_session)(handle)
    }

    pub unsafe fn destroy_session(&self, handle: NvDRSSessionHandle) -> NvAPI_Status {
        (self.destroy_session)(handle)
    }

    pub unsafe fn load_settings(&self, handle: NvDRSSessionHandle) -> NvAPI_Status {
        (self.load_settings)(handle)
    }

    pub unsafe fn save_settings(&self, handle: NvDRSSessionHandle) -> NvAPI_Status {
        (self.save_settings)(handle)
    }

    pub unsafe fn get_num_profiles(&self, handle: NvDRSSessionHandle, count: *mut u32) -> NvAPI_Status {
        (self.get_num_profiles)(handle, count)
    }

    pub unsafe fn enum_profiles(&self, handle: NvDRSSessionHandle, index: u32, profile: *mut NvDRSProfileHandle) -> NvAPI_Status {
        (self.enum_profiles)(handle, index, profile)
    }

    pub unsafe fn get_profile_info(&self, handle: NvDRSSessionHandle, profile: NvDRSProfileHandle, info: *mut NVDRS_PROFILE) -> NvAPI_Status {
        (self.get_profile_info)(handle, profile, info)
    }

    pub unsafe fn enum_applications(&self, handle: NvDRSSessionHandle, profile: NvDRSProfileHandle, start: u32, count: *mut u32, apps: *mut NVDRS_APPLICATION) -> NvAPI_Status {
        (self.enum_applications)(handle, profile, start, count, apps)
    }

    pub unsafe fn get_base_profile(&self, handle: NvDRSSessionHandle, profile: *mut NvDRSProfileHandle) -> NvAPI_Status {
        (self.get_base_profile)(handle, profile)
    }

    pub unsafe fn find_profile_by_name(&self, handle: NvDRSSessionHandle, name: *const u16, profile: *mut NvDRSProfileHandle) -> NvAPI_Status {
        (self.find_profile_by_name)(handle, name, profile)
    }

    pub unsafe fn get_setting(&self, handle: NvDRSSessionHandle, profile: NvDRSProfileHandle, setting_id: u32, setting: *mut NVDRS_SETTING) -> NvAPI_Status {
        (self.get_setting)(handle, profile, setting_id, setting)
    }

    pub unsafe fn set_setting(&self, handle: NvDRSSessionHandle, profile: NvDRSProfileHandle, setting: *const NVDRS_SETTING) -> NvAPI_Status {
        (self.set_setting)(handle, profile, setting)
    }

    pub unsafe fn create_profile(&self, handle: NvDRSSessionHandle, info: *mut NVDRS_PROFILE, profile: *mut NvDRSProfileHandle) -> NvAPI_Status {
        (self.create_profile)(handle, info, profile)
    }

    pub unsafe fn create_application(&self, handle: NvDRSSessionHandle, profile: NvDRSProfileHandle, app: *mut NVDRS_APPLICATION) -> NvAPI_Status {
        (self.create_application)(handle, profile, app)
    }

    pub unsafe fn delete_profile(&self, handle: NvDRSSessionHandle, profile: NvDRSProfileHandle) -> NvAPI_Status {
        (self.delete_profile)(handle, profile)
    }

    pub unsafe fn delete_application(&self, handle: NvDRSSessionHandle, profile: NvDRSProfileHandle, app_name: *const u16) -> NvAPI_Status {
        (self.delete_application)(handle, profile, app_name)
    }

    pub unsafe fn delete_profile_setting(&self, handle: NvDRSSessionHandle, profile: NvDRSProfileHandle, setting_id: u32) -> NvAPI_Status {
        (self.delete_profile_setting)(handle, profile, setting_id)
    }
}

/// Convert wide string to Rust String
pub fn wstring_to_string(ws: &[u16]) -> String {
    let len = ws.iter().position(|&c| c == 0).unwrap_or(ws.len());
    String::from_utf16_lossy(&ws[..len])
}

/// Convert Rust string to wide string buffer
pub fn string_to_wstring(s: &str) -> NvAPI_UnicodeString {
    let mut buf: NvAPI_UnicodeString = [0u16; NVAPI_UNICODE_STRING_MAX];
    for (i, c) in s.encode_utf16().take(NVAPI_UNICODE_STRING_MAX - 1).enumerate() {
        buf[i] = c;
    }
    buf
}

/// DRS Session wrapper with RAII cleanup
pub struct DrsSession {
    api: DrsApi,
    handle: NvDRSSessionHandle,
}

impl DrsSession {
    /// Create a new DRS session and load settings
    pub fn new() -> Option<Self> {
        let api = DrsApi::load()?;
        
        unsafe {
            let mut handle: NvDRSSessionHandle = ptr::null_mut();
            
            if api.create_session(&mut handle) != NVAPI_OK {
                return None;
            }
            
            if api.load_settings(handle) != NVAPI_OK {
                api.destroy_session(handle);
                return None;
            }
            
            Some(Self { api, handle })
        }
    }

    /// Get number of profiles
    pub fn get_num_profiles(&self) -> Option<u32> {
        unsafe {
            let mut count = 0u32;
            if self.api.get_num_profiles(self.handle, &mut count) == NVAPI_OK {
                Some(count)
            } else {
                None
            }
        }
    }

    /// Enumerate all profiles
    pub fn enumerate_profiles(&self) -> Vec<(NvDRSProfileHandle, NVDRS_PROFILE)> {
        let mut profiles = Vec::new();
        
        unsafe {
            let mut index = 0u32;
            loop {
                let mut profile_handle: NvDRSProfileHandle = ptr::null_mut();
                
                let status = self.api.enum_profiles(self.handle, index, &mut profile_handle);
                if status != NVAPI_OK {
                    break;
                }
                
                let mut profile_info = NVDRS_PROFILE::default();
                if self.api.get_profile_info(self.handle, profile_handle, &mut profile_info) == NVAPI_OK {
                    profiles.push((profile_handle, profile_info));
                }
                
                index += 1;
            }
        }
        
        profiles
    }

    /// Get applications for a profile
    pub fn get_applications(&self, profile_handle: NvDRSProfileHandle) -> Vec<NVDRS_APPLICATION> {
        let mut apps = Vec::new();
        
        unsafe {
            let mut start = 0u32;
            let mut count = 1u32;
            
            while count > 0 {
                let mut app = NVDRS_APPLICATION::default();
                count = 1;
                
                let status = self.api.enum_applications(self.handle, profile_handle, start, &mut count, &mut app);
                if status != NVAPI_OK || count == 0 {
                    break;
                }
                
                apps.push(app);
                start += 1;
            }
        }
        
        apps
    }

    /// Find profile by name
    pub fn find_profile(&self, name: &str) -> Option<NvDRSProfileHandle> {
        unsafe {
            let wname = string_to_wstring(name);
            let mut handle: NvDRSProfileHandle = ptr::null_mut();
            
            if self.api.find_profile_by_name(self.handle, wname.as_ptr(), &mut handle) == NVAPI_OK {
                Some(handle)
            } else {
                None
            }
        }
    }

    /// Get a setting value
    pub fn get_setting(&self, profile_handle: NvDRSProfileHandle, setting_id: u32) -> Option<NVDRS_SETTING> {
        unsafe {
            let mut setting = NVDRS_SETTING::default();
            
            if self.api.get_setting(self.handle, profile_handle, setting_id, &mut setting) == NVAPI_OK {
                Some(setting)
            } else {
                None
            }
        }
    }

    /// Set a setting value (DWORD). Returns the raw NVAPI status (0 = OK).
    pub fn set_setting_dword_status(&self, profile_handle: NvDRSProfileHandle, setting_id: u32, value: u32) -> NvAPI_Status {
        unsafe {
            let mut setting = NVDRS_SETTING::default();
            setting.setting_id = setting_id;
            setting.setting_type = NVDRS_SETTING_TYPE::NVDRS_DWORD_TYPE as u32;
            setting.current_value.u32_value = value;

            self.api.set_setting(self.handle, profile_handle, &setting)
        }
    }

    /// Set a setting value (DWORD). Convenience wrapper returning a bool.
    pub fn set_setting_dword(&self, profile_handle: NvDRSProfileHandle, setting_id: u32, value: u32) -> bool {
        self.set_setting_dword_status(profile_handle, setting_id, value) == NVAPI_OK
    }

    /// Save settings to driver
    pub fn save(&self) -> bool {
        unsafe {
            self.api.save_settings(self.handle) == NVAPI_OK
        }
    }

    /// Create a new profile
    pub fn create_profile(&self, name: &str) -> Option<NvDRSProfileHandle> {
        unsafe {
            let mut info = NVDRS_PROFILE::default();
            info.profile_name = string_to_wstring(name);
            
            let mut handle: NvDRSProfileHandle = ptr::null_mut();
            
            if self.api.create_profile(self.handle, &mut info, &mut handle) == NVAPI_OK {
                Some(handle)
            } else {
                None
            }
        }
    }

    /// Add application to profile
    pub fn add_application(&self, profile_handle: NvDRSProfileHandle, app_name: &str) -> bool {
        unsafe {
            let mut app = NVDRS_APPLICATION::default();
            app.app_name = string_to_wstring(app_name);
            app.user_friendly_name = string_to_wstring(app_name);

            self.api.create_application(self.handle, profile_handle, &mut app) == NVAPI_OK
        }
    }

    /// Delete an entire profile.
    pub fn delete_profile(&self, profile_handle: NvDRSProfileHandle) -> NvAPI_Status {
        unsafe { self.api.delete_profile(self.handle, profile_handle) }
    }

    /// Remove an application entry from a profile. `app_name` is the exe name
    /// (case-insensitive in practice, e.g. "eldenring.exe").
    pub fn delete_application(&self, profile_handle: NvDRSProfileHandle, app_name: &str) -> NvAPI_Status {
        let wname = string_to_wstring(app_name);
        unsafe { self.api.delete_application(self.handle, profile_handle, wname.as_ptr()) }
    }

    /// Clear a single setting on a profile (reverts to driver default).
    pub fn delete_profile_setting(&self, profile_handle: NvDRSProfileHandle, setting_id: u32) -> NvAPI_Status {
        unsafe { self.api.delete_profile_setting(self.handle, profile_handle, setting_id) }
    }
}

impl Drop for DrsSession {
    fn drop(&mut self) {
        unsafe {
            self.api.destroy_session(self.handle);
        }
    }
}
