use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::ptr::null_mut;
use winapi::ctypes::c_void;
use winapi::shared::minwindef::{DWORD, LPVOID, PUINT};
use winapi::um::fileapi::{GetFileAttributesExW, WIN32_FILE_ATTRIBUTE_DATA};
use winapi::um::minwinbase::GetFileExInfoStandard;
use winapi::um::winnt::WCHAR;
use winapi::um::winver::{GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW};

fn get_version_info_field(file_path: &str, field: &str) -> Option<(String, u64)> {
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

fn main() {
    let path = r"C:\Windows\System32\DriverStore\FileRepository\realtekservice.inf_amd64_5585cd55bed009c8\RtkAudUService64.exe";
    let company = get_version_info_field(path, "CompanyName")
        .map(|v| v.0)
        .unwrap_or_default();
    let description = get_version_info_field(path, "FileDescription")
        .map(|v| v.0)
        .unwrap_or_default();
    let version = get_version_info_field(path, "FileVersion")
        .map(|v| v.0)
        .unwrap_or_default();
    let size = get_version_info_field(path, "CompanyName")
        .map(|v| v.1)
        .unwrap_or(0);

    println!("CompanyName : {}", company);
    println!("Description : {}", description);
    println!("Version     : {}", version);
    println!("File size   : {} bytes", size);
}
