use std::os::raw::c_uint;

pub type HANDLE = *mut winapi::ctypes::c_void;
pub type DWORD = u32;
pub type BOOL = i32;

#[link(name = "kernel32")]
unsafe extern "system" {
    pub fn LocalFree(hMem: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    pub fn GetDriveTypeA(lpRootPathName: *const i8) -> c_uint;
    pub fn OpenProcess(dwDesiredAccess: DWORD, bInheritHandle: BOOL, dwProcessId: DWORD) -> HANDLE;
    pub fn CloseHandle(hObject: HANDLE) -> BOOL;
    //    pub fn GetLastError() -> DWORD;
}
