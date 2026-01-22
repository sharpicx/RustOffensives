use winapi::ctypes::c_void;
use winapi::shared::minwindef::ULONG;
use winapi::shared::ntdef::{NTSTATUS, OBJECT_ATTRIBUTES};
use winapi::um::winnt::{ACCESS_MASK, CONTEXT, HANDLE, PVOID};

#[allow(non_snake_case)]
#[repr(C)]
pub struct CLIENT_ID {
    pub UniqueProcess: *mut c_void,
    pub UniqueThread: *mut c_void,
}

#[repr(C)]
pub struct PsProtection {
    pub r#type: u8,
    pub r#signer: u8,
    pub r#audit: u8,
}

#[repr(C)]
pub struct ProcessBasicInformation {
    pub reserved1: *mut c_void,
    pub peb_base_address: *mut c_void,
    pub reserved2: [*mut c_void; 2],
    pub unique_process_id: usize,
    pub inherited_from_unique_process_id: usize,
}

pub struct ProtectionValue(pub i32);

impl std::fmt::Display for ProtectionValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self.0 {
            0 => "None",

            11 => "Authenticode-Light",
            21 => "CodeGen-Light",
            31 => "Antimalware-Light",
            41 => "LSA-Light",
            51 => "Windows-Light",
            61 => "WinTcb-Light",
            71 => "Max-Light",

            12 => "Authenticode",
            22 => "CodeGen",
            32 => "Antimalware",
            42 => "LSA",
            52 => "Windows",
            62 => "WinTcb",
            72 => "Max",

            13 => "Authenticode-Max",
            23 => "CodeGen-Max",
            33 => "Antimalware-Max",
            43 => "LSA-Max",
            53 => "Windows-Max",
            63 => "WinTcb-Max",
            73 => "Max-Max",

            _ => "Unknown",
        };

        write!(f, "{}", s)
    }
}

unsafe extern "system" {
    pub fn NtOpenProcess(
        ProcessHandle: *mut HANDLE,
        DesiredAccess: u32,
        ObjectAttributes: *mut OBJECT_ATTRIBUTES,
        ClientId: *const CLIENT_ID,
    ) -> NTSTATUS;
    pub fn NtProtectVirtualMemory(
        ProcessHandle: HANDLE,
        BaseAddress: *mut PVOID,
        RegionSize: *mut usize,
        NewProtect: ULONG,
        OldProtect: *mut ULONG,
    ) -> NTSTATUS;
    pub fn NtWriteVirtualMemory(
        ProcessHandle: HANDLE,
        BaseAddress: *mut c_void,
        Buffer: *const c_void,
        NumberOfBytesToWrite: usize,
        NumberOfBytesWritten: *mut usize,
    ) -> NTSTATUS;
    pub fn NtGetContextThread(thread_handle: HANDLE, thread_context: *mut CONTEXT) -> ULONG;
    pub fn NtSetContextThread(thread_handle: HANDLE, thread_context: *mut CONTEXT) -> ULONG;
    pub fn NtQuerySystemInformation(
        SystemInformationClass: ULONG,
        SystemInformation: *mut c_void,
        SystemInformationLength: ULONG,
        ReturnLength: *mut ULONG,
    ) -> NTSTATUS;
    pub fn NtQueryInformationProcess(
        ProcessHandle: HANDLE,
        ProcessInformationClass: ULONG,
        ProcessInformation: *mut c_void,
        ProcessInformationLength: ULONG,
        ReturnLength: *mut ULONG,
    ) -> NTSTATUS;
    pub fn NtOpenThread(
        ThreadHandle: *mut HANDLE,
        DesiredAccess: ACCESS_MASK,
        ObjectAttributes: *const OBJECT_ATTRIBUTES,
        ClientId: *const CLIENT_ID,
    ) -> NTSTATUS;
    pub fn NtClose(Handle: HANDLE) -> NTSTATUS;
}
