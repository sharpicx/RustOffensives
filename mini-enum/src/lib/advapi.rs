use core::ffi::{c_int, c_void};
use windows_alt::core::PCWSTR;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CREDENTIALW {
    pub flags: u32,
    pub cred_type: u32,
    pub target_name: *mut u16,
    pub comment: *mut u16,
    pub last_written: i64,
    pub credential_blob_size: u32,
    pub credential_blob: *mut u8,
    pub persist: u32,
    pub attribute_count: u32,
    pub attributes: *mut (),
    pub target_alias: *mut u16,
    pub user_name: *mut u16,
}

#[repr(i32)]
pub enum TagInfoLevel {
    ETagInfoLevelNameFromTag = 1,
    ETagInfoLevelNamesReferencingModule = 2,
    ETagInfoLevelNameTagMapping = 3,
    ETagInfoLevelMax = 4,
}

#[repr(C)]
pub struct ScServiceTagQuery {
    pub process_id: u32,
    pub service_tag: u32,
    pub unknown: u32,
    pub buffer: *mut u16,
}

#[link(name = "advapi32.dll", kind = "raw-dylib", modifiers = "+verbatim")]
unsafe extern "system" {
    pub fn I_QueryTagInformation(
        MachineName: PCWSTR,
        InfoLevel: TagInfoLevel,
        TagInfo: *mut std::ffi::c_void,
    ) -> u32;
    pub fn CredEnumerateW(
        filter: *const u16,
        flags: u32,
        count: *mut u32,
        creds: *mut *mut *mut CREDENTIALW,
    ) -> i32;

    pub fn CredFree(buffer: *mut core::ffi::c_void);
    pub fn IsTextUnicode(buf: *const c_void, size: c_int, flags: *mut c_int) -> i32;
}
