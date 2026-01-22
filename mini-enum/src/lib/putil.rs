use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::ptr::null_mut;
use winapi::ctypes::c_void;
use winapi::shared::minwindef::ULONG;
use winapi::shared::minwindef::{DWORD, FALSE};
use winapi::shared::minwindef::{LPVOID, PUINT};
use winapi::shared::sddl::{
    ConvertSecurityDescriptorToStringSecurityDescriptorW, ConvertSidToStringSidW, SDDL_REVISION_1,
};
use winapi::shared::winerror::ERROR_SUCCESS;
use winapi::um::accctrl::{SE_FILE_OBJECT, SE_OBJECT_TYPE};
use winapi::um::aclapi::GetNamedSecurityInfoW;
use winapi::um::fileapi::{GetFileAttributesExW, WIN32_FILE_ATTRIBUTE_DATA};
use winapi::um::minwinbase::GetFileExInfoStandard;
use winapi::um::securitybaseapi::GetSecurityDescriptorOwner;
use winapi::um::winbase::QueryFullProcessImageNameW;
use winapi::um::winnt::WCHAR;
use winapi::um::winnt::{
    DACL_SECURITY_INFORMATION, OWNER_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR, PSID, PWSTR,
};
use winapi::um::winver::{GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW};

use crate::lib::kernel32::{CloseHandle, LocalFree, OpenProcess};
use crate::lib::ntdll::{NtQueryInformationProcess, ProcessBasicInformation, PsProtection};

pub const PROCESS_QUERY_LIMITED_INFORMATION: DWORD = 0x1000;
pub const PROCESS_PROTECTION_INFORMATION: ULONG = 61;

#[derive(Debug, Default)]
pub struct SecurityInfos {
    pub owner: Option<String>,
    pub sddl: Option<String>,
    pub security_descriptor: Option<PSECURITY_DESCRIPTOR>,
}

// https://github.com/sharpicx/RustOffensives/tree/main/getversioninfo-rs
pub fn get_version_info_field(file_path: &str, field: &str) -> Option<(String, u64)> {
    unsafe {
        let file_w: Vec<u16> = file_path.encode_utf16().chain(Some(0)).collect();
        let mut file_data: WIN32_FILE_ATTRIBUTE_DATA = std::mem::zeroed();
        if GetFileAttributesExW(
            file_w.as_ptr(),
            GetFileExInfoStandard,
            &mut file_data as *mut _ as *mut _,
        ) == 0
        {
            return None;
        }
        let file_size = ((file_data.nFileSizeHigh as u64) << 32) | file_data.nFileSizeLow as u64;
        let mut dummy: DWORD = 0;
        let size: DWORD = GetFileVersionInfoSizeW(file_w.as_ptr(), &mut dummy);
        if size == 0 {
            return None;
        }

        let mut buffer = vec![0u8; size as usize];
        if GetFileVersionInfoW(file_w.as_ptr(), 0, size, buffer.as_mut_ptr() as *mut c_void) == 0 {
            return None;
        }
        let mut lp_translate: LPVOID = null_mut();
        let mut len: DWORD = 0;
        if VerQueryValueW(
            buffer.as_ptr() as *const c_void,
            "\\VarFileInfo\\Translation\0"
                .encode_utf16()
                .collect::<Vec<u16>>()
                .as_ptr(),
            &mut lp_translate,
            &mut len as PUINT,
        ) == 0
        {
            return None;
        }
        if len < 4 {
            return None;
        }
        let trans = std::slice::from_raw_parts(lp_translate as *const u16, 2);
        let lang = trans[0];
        let codepage = trans[1];
        let query = format!("\\StringFileInfo\\{:04x}{:04x}\\{}", lang, codepage, field);
        let query_w: Vec<u16> = query.encode_utf16().chain(Some(0)).collect();
        let mut lp_value: LPVOID = null_mut();
        let mut len: DWORD = 0;
        if VerQueryValueW(
            buffer.as_ptr() as *const c_void,
            query_w.as_ptr(),
            &mut lp_value,
            &mut len as PUINT,
        ) == 0
        {
            return None;
        }
        if len == 0 {
            return None;
        }
        let slice = std::slice::from_raw_parts(lp_value as *const WCHAR, len as usize - 1);
        let field_value = OsString::from_wide(slice).to_string_lossy().into_owned();
        Some((field_value, file_size))
    }
}

pub fn get_security_infos(object_name: &str, object_type: SE_OBJECT_TYPE) -> SecurityInfos {
    unsafe {
        let mut infos = SecurityInfos::default();
        let name_w: Vec<u16> = object_name.encode_utf16().chain(Some(0)).collect();
        let mut sd = null_mut();
        let status = GetNamedSecurityInfoW(
            name_w.as_ptr(),
            object_type,
            OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION,
            null_mut(),
            null_mut(),
            null_mut(),
            null_mut(),
            &mut sd,
        );
        if status != ERROR_SUCCESS {
            return infos;
        }
        infos.security_descriptor = Some(sd);
        let mut sddl_ptr: PWSTR = null_mut();
        let ok = ConvertSecurityDescriptorToStringSecurityDescriptorW(
            sd,
            SDDL_REVISION_1 as u32,
            OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION,
            &mut sddl_ptr,
            null_mut(),
        );
        if ok != 0 {
            let mut len = 0;
            while *sddl_ptr.add(len) != 0 {
                len += 1;
            }
            let sddl = OsString::from_wide(std::slice::from_raw_parts(sddl_ptr, len))
                .to_string_lossy()
                .into_owned();
            infos.sddl = Some(sddl);
            LocalFree(sddl_ptr as _);
        }
        let mut owner_sid: PSID = null_mut();
        let mut owner_defaulted = 0;
        if GetSecurityDescriptorOwner(sd, &mut owner_sid, &mut owner_defaulted) != 0 {
            let mut sid_string: PWSTR = null_mut();
            if ConvertSidToStringSidW(owner_sid, &mut sid_string) != 0 {
                let mut len = 0;
                while *sid_string.add(len) != 0 {
                    len += 1;
                }
                let owner = OsString::from_wide(std::slice::from_raw_parts(sid_string, len))
                    .to_string_lossy()
                    .into_owned();

                infos.owner = Some(owner);
                LocalFree(sid_string as _);
            }
        }
        LocalFree(sd as _);
        infos
    }
}

pub fn get_file_sddl(path: &str) -> Option<String> {
    unsafe {
        let path_w: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();
        let mut sd: PSECURITY_DESCRIPTOR = null_mut();
        let status = GetNamedSecurityInfoW(
            path_w.as_ptr(),
            SE_FILE_OBJECT,
            OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION,
            null_mut(),
            null_mut(),
            null_mut(),
            null_mut(),
            &mut sd,
        );
        if status != ERROR_SUCCESS {
            return None;
        }
        let mut sddl_ptr: PWSTR = null_mut();
        let ok = ConvertSecurityDescriptorToStringSecurityDescriptorW(
            sd,
            SDDL_REVISION_1 as u32,
            OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION,
            &mut sddl_ptr,
            null_mut(),
        );
        if ok == FALSE {
            LocalFree(sd as _);
            return None;
        }
        let sddl = {
            let mut len = 0;
            while *sddl_ptr.add(len) != 0 {
                len += 1;
            }
            String::from_utf16_lossy(std::slice::from_raw_parts(sddl_ptr, len))
        };
        LocalFree(sddl_ptr as _);
        LocalFree(sd as _);
        Some(sddl)
    }
}

pub fn get_parent_pid(pid: u32) -> Option<u32> {
    unsafe {
        let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if h.is_null() {
            return None;
        }
        let mut pbi: ProcessBasicInformation = std::mem::zeroed();
        let mut ret_len: ULONG = 0;
        let status = NtQueryInformationProcess(
            h,
            0, // ProcessBasicInformation
            &mut pbi as *mut _ as *mut c_void,
            std::mem::size_of::<ProcessBasicInformation>() as ULONG,
            &mut ret_len,
        );
        CloseHandle(h);
        if status != 0 {
            None
        } else {
            Some(pbi.inherited_from_unique_process_id as u32)
        }
    }
}

pub fn get_process_name(pid: u32) -> Option<String> {
    unsafe {
        let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if h.is_null() {
            return None;
        }
        let mut buf = [0u16; 260];
        let mut size: u32 = buf.len() as u32;

        let ok = QueryFullProcessImageNameW(h, 0, buf.as_mut_ptr(), &mut size);

        CloseHandle(h);

        if ok == 0 {
            return None;
        }
        Some(String::from_utf16_lossy(&buf[..size as usize]))
    }
}

pub fn get_process_protection_info(pid: u32) -> Option<i32> {
    unsafe {
        let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if h.is_null() {
            return None;
        }
        let mut ps = PsProtection {
            r#type: 0,
            r#signer: 0,
            r#audit: 0,
        };
        let mut ret_len: u32 = 0;
        let status = NtQueryInformationProcess(
            h,
            PROCESS_PROTECTION_INFORMATION,
            &mut ps as *mut _ as *mut c_void,
            std::mem::size_of::<PsProtection>() as ULONG,
            &mut ret_len as *mut ULONG,
        );
        if status != 0 {
            CloseHandle(h);
            return None;
        }
        let value = (ps.r#type as i32) | (ps.r#audit as i32) | ((ps.r#signer as i32) << 4);
        CloseHandle(h);
        Some(value)
    }
}
