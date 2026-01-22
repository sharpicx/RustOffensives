use regex::Regex;
use std::collections::HashMap;
use std::path::Path;
use winreg::RegKey;
use winreg::enums::*;
use wmi::{Variant, WMIConnection};

use crate::lib::fileutil::is_dotnet_assembly;
use crate::lib::putil::{get_file_sddl, get_security_infos, get_version_info_field};

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

pub fn enum_windows_services() -> Result<(), Box<dyn std::error::Error>> {
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
// https://redfoxsecurity.medium.com/abusing-acl-misconfigurations-e1f7a7dea14d
