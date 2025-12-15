use regex::Regex;
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::os::windows::ffi::OsStringExt;
use std::path::Path;
use std::ptr::null_mut;
use winapi::ctypes::c_void;
use winapi::shared::minwindef::{DWORD, FALSE, LPVOID, PUINT};
use winapi::shared::sddl::{
    ConvertSecurityDescriptorToStringSecurityDescriptorW, ConvertSidToStringSidW, SDDL_REVISION_1,
};
use winapi::shared::winerror::ERROR_SUCCESS;
use winapi::um::accctrl::{SE_FILE_OBJECT, SE_OBJECT_TYPE};
use winapi::um::aclapi::GetNamedSecurityInfoW;
use winapi::um::fileapi::{GetFileAttributesExW, WIN32_FILE_ATTRIBUTE_DATA};
use winapi::um::minwinbase::GetFileExInfoStandard;
use winapi::um::securitybaseapi::GetSecurityDescriptorOwner;
use winapi::um::winbase::LocalFree;
use winapi::um::winnt::{
    DACL_SECURITY_INFORMATION, OWNER_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR, PSID, PWSTR, WCHAR,
};
use winapi::um::winver::{GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW};
use winreg::RegKey;
use winreg::enums::*;
use wmi::{Variant, WMIConnection};

#[derive(Debug, Default)]
struct SecurityInfos {
    owner: Option<String>,
    sddl: Option<String>,
    security_descriptor: Option<PSECURITY_DESCRIPTOR>,
}

struct ServiceInfo {
    service_name: String,
    binary_path: Option<String>,
    company_name: Option<String>,
    description: Option<String>,
    version: Option<String>,
    is_dotnet: Option<bool>,
    binary_path_sddl: Option<String>,
    service_sddl: Option<String>,
}

fn get_security_infos(object_name: &str, object_type: SE_OBJECT_TYPE) -> SecurityInfos {
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

fn get_file_sddl(path: &str) -> Option<String> {
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

// https://github.com/sharpicx/RustOffensives/tree/main/getversioninfo-rs
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

// https://github.com/sharpicx/EFEW/blob/main/src/lib/fileutil.rs
fn is_dotnet_assembly(file_name: &str) -> bool {
    let file = File::open(file_name);
    if file.is_err() {
        return false;
    }
    let mut file = file.unwrap();
    let len = file.metadata();
    if len.is_err() {
        return false;
    }
    let file_len = len.unwrap().len();
    if file_len < 64 {
        return false;
    }
    if file.seek(SeekFrom::Start(0x3C)).is_err() {
        return false;
    }
    let mut buf4 = [0u8; 4];
    if file.read_exact(&mut buf4).is_err() {
        return false;
    }
    let mut pe_header_ptr = u32::from_le_bytes(buf4) as u64;
    if pe_header_ptr == 0 {
        pe_header_ptr = 0x80;
    }
    if pe_header_ptr > file_len.saturating_sub(256) {
        return false;
    }
    if file.seek(SeekFrom::Start(pe_header_ptr)).is_err() {
        return false;
    }
    if file.read_exact(&mut buf4).is_err() {
        return false;
    }
    let pe_sig = u32::from_le_bytes(buf4);
    if pe_sig != 0x0000_4550 {
        return false;
    }
    if file.seek(SeekFrom::Current(20)).is_err() {
        return false;
    }
    let mut buf2 = [0u8; 2];
    if file.read_exact(&mut buf2).is_err() {
        return false;
    }
    let pe_format = u16::from_le_bytes(buf2);
    const PE32: u16 = 0x10b;
    const PE32_PLUS: u16 = 0x20b;

    if pe_format != PE32 && pe_format != PE32_PLUS {
        return false;
    }
    let data_dir_offset = if pe_format == PE32 { 232 } else { 248 };
    let cli_rva_pos = pe_header_ptr + data_dir_offset;
    if file.seek(SeekFrom::Start(cli_rva_pos)).is_err() {
        return false;
    }
    if file.read_exact(&mut buf4).is_err() {
        return false;
    }
    let cli_header_rva = u32::from_le_bytes(buf4);
    cli_header_rva != 0
}

fn get_service_command(row: &HashMap<String, Variant>) -> Option<String> {
    if let Some(Variant::String(path)) = row.get("PathName") {
        let trimmed = path.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    if let Some(Variant::String(name)) = row.get("Name") {
        return get_service_command_from_registry(name);
    }
    None
}

fn get_service_command_from_registry(service_name: &str) -> Option<String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let path = format!(r"SYSTEM\CurrentControlSet\Services\{}", service_name);
    let key = hklm.open_subkey(path).ok()?;
    let image_path: String = key.get_value("ImagePath").ok()?;
    Some(image_path)
}

fn get_service_binary_path(command: &str) -> Option<String> {
    let re = Regex::new(r"(?i)^\W*([a-z]:\\.+?(\.exe|\.dll|\.sys))\W*").ok()?;
    let caps = re.captures(command)?;
    Some(caps.get(1)?.as_str().to_string())
}

fn get_service_dll(service_name: &str) -> Option<String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let path1 = format!(
        r"SYSTEM\CurrentControlSet\Services\{}\Parameters",
        service_name
    );
    let key1 = hklm.open_subkey(&path1);
    if key1.is_ok() {
        let key1 = key1.unwrap();
        let val1 = key1.get_value::<String, _>("ServiceDll");
        if val1.is_ok() {
            let val1 = val1.unwrap();
            return Some(val1);
        }
    }
    let path2 = format!(r"SYSTEM\CurrentControlSet\Services\{}", service_name);
    let key2 = hklm.open_subkey(&path2);
    if key2.is_ok() {
        let key2 = key2.unwrap();
        let val2 = key2.get_value::<String, _>("ServiceDll");
        if val2.is_ok() {
            let val2 = val2.unwrap();
            return Some(val2);
        }
    }
    None
}

fn get_service_sddl(service_name: Option<&str>) -> Option<String> {
    match service_name {
        Some(name) => get_security_infos(name, winapi::um::accctrl::SE_SERVICE).sddl,
        None => None,
    }
}

fn wmi_query(q: &str) -> Result<Vec<HashMap<String, Variant>>, Box<dyn std::error::Error>> {
    let con = WMIConnection::with_namespace_path("ROOT\\CIMV2")?;
    let rows: Vec<HashMap<String, Variant>> = con.raw_query(q)?;
    Ok(rows)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rows = wmi_query("SELECT * FROM Win32_Service")?;
    let mut services: Vec<ServiceInfo> = Vec::new();

    for row in rows {
        let service_name: String;
        if let Some(Variant::String(name)) = row.get("Name") {
            service_name = name.clone();
        } else {
            continue;
        }
        let mut binary_path: Option<String> = None;
        let mut company_name: Option<String> = None;
        let mut description: Option<String> = None;
        let mut version: Option<String> = None;
        let mut is_dotnet: Option<bool> = None;
        let mut binary_path_sddl: Option<String> = None;
        let service_command = get_service_command(&row);
        let mut service_sddl: Option<String> = None;
        if let Some(service_command) = service_command {
            binary_path = get_service_binary_path(&service_command);
            let service_dll = Some(service_name.as_str()).and_then(get_service_dll);
            if binary_path.is_some() && service_dll.is_some() {
                let bp = binary_path.as_ref().unwrap();
                if bp.to_lowercase().ends_with(r"\svchost.exe") {
                    binary_path = service_dll;
                }
            }
            if let Some(bp) = binary_path.as_ref() {
                if !bp.is_empty() && Path::new(bp).exists() {
                    if let Some((v, _)) = get_version_info_field(bp, "CompanyName") {
                        company_name = Some(v);
                    }
                    if let Some((v, _)) = get_version_info_field(bp, "FileDescription") {
                        description = Some(v);
                    }
                    if let Some((v, _)) = get_version_info_field(bp, "FileVersion") {
                        version = Some(v);
                    }
                    is_dotnet = Some(is_dotnet_assembly(bp));
                    binary_path_sddl = get_file_sddl(bp);
                }
            }
            service_sddl = get_service_sddl(Some(&service_name));
        }

        services.push(ServiceInfo {
            service_name,
            binary_path,
            company_name,
            description,
            version,
            is_dotnet,
            binary_path_sddl,
            service_sddl,
        });
    }
    for service in &services {
        println!("── Service: {}", service.service_name);
        if let Some(bp) = &service.binary_path {
            println!("   ├── Binary Path: {}", bp);
        }
        if let Some(cn) = &service.company_name {
            println!("   ├── Company Name: {}", cn);
        }
        if let Some(desc) = &service.description {
            println!("   ├── Description: {}", desc);
        }
        if let Some(ver) = &service.version {
            println!("   ├── Version: {}", ver);
        }
        if let Some(dotnet) = service.is_dotnet {
            println!("   ├── Is .NET: {}", dotnet);
        }
        if let Some(sddl) = &service.binary_path_sddl {
            println!("   ├── Binary SDDL: {}", sddl);
        }
        if let Some(sddl) = &service.service_sddl {
            println!("   └── Service SDDL: {}", sddl);
        } else {
            println!("   └── Service SDDL: <None>");
        }

        println!();
    }

    Ok(())
}

// https://redfoxsec.com/blog/abusing-acl-misconfigurations/
