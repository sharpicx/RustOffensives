use chrono::{DateTime, Local};
use colored::*;
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use widestring::U16CString;
use windows::Win32::Foundation::ERROR_INSUFFICIENT_BUFFER;
use windows::Win32::NetworkManagement::IpHelper::{
    GetExtendedUdpTable, MIB_UDPROW_OWNER_MODULE, MIB_UDPTABLE_OWNER_MODULE, UDP_TABLE_OWNER_MODULE,
};
use windows::Win32::Networking::WinSock::AF_INET;
use windows::core::PCWSTR;
use wmi::Variant;
use wmi::WMIConnection;

#[repr(i32)]
enum TagInfoLevel {
    ETagInfoLevelNameFromTag = 1,
    ETagInfoLevelNamesReferencingModule = 2,
    ETagInfoLevelNameTagMapping = 3,
    ETagInfoLevelMax = 4,
}

#[repr(C)]
struct ScServiceTagQuery {
    process_id: u32,
    service_tag: u32,
    unknown: u32,
    buffer: *mut u16,
}

#[link(name = "advapi32.dll", kind = "raw-dylib", modifiers = "+verbatim")]
unsafe extern "system" {
    fn I_QueryTagInformation(
        MachineName: PCWSTR,
        InfoLevel: TagInfoLevel,
        TagInfo: *mut std::ffi::c_void,
    ) -> u32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn LocalFree(hMem: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
}

fn get_service_name_from_tag(process_id: u32, service_tag: u32) -> Option<String> {
    let _ = TagInfoLevel::ETagInfoLevelMax;
    let _ = TagInfoLevel::ETagInfoLevelNamesReferencingModule;
    let _ = TagInfoLevel::ETagInfoLevelNameTagMapping;
    let mut query = ScServiceTagQuery {
        process_id: process_id,
        service_tag: service_tag,
        unknown: 0,
        buffer: std::ptr::null_mut(),
    };

    let status = unsafe {
        I_QueryTagInformation(
            PCWSTR::null(),
            TagInfoLevel::ETagInfoLevelNameFromTag,
            &mut query as *mut _ as *mut std::ffi::c_void,
        )
    };

    if status == 0 {
        let name = unsafe { U16CString::from_ptr_str(query.buffer) }
            .to_string()
            .ok();
        unsafe { LocalFree(query.buffer as *mut _) };
        return name;
    }

    None
}

fn filetime_to_systemtime(ft: i64) -> SystemTime {
    const EPOCH_DIFFERENCE_100NS: i64 = 116444736000000000;

    let ft = ft as i128;
    let unix_100ns = ft - EPOCH_DIFFERENCE_100NS as i128;
    let unix_secs = unix_100ns / 10_000_000;
    let unix_sub_100ns = unix_100ns % 10_000_000;
    let nanos = (unix_sub_100ns * 100) as u64;

    UNIX_EPOCH + Duration::new(unix_secs as u64, nanos as u32)
}

fn fmt_local(t: SystemTime) -> String {
    let dt: DateTime<Local> = t.into();
    dt.format("%A, %d-%m-%Y %I:%M %p").to_string()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut process_names: HashMap<String, String> = HashMap::new();
    let mut process_command_lines: HashMap<String, String> = HashMap::new();
    let wmi_con = WMIConnection::new()?;
    let results: Vec<HashMap<String, Variant>> =
        wmi_con.raw_query("SELECT * FROM Win32_Process")?;
    for process in results {
        if let (Some(pid), Some(name)) = (process.get("ProcessId"), process.get("Name")) {
            let pid_str = match pid {
                Variant::UI4(v) => v.to_string(),
                Variant::I4(v) => v.to_string(),
                _ => continue,
            };
            let name_str = match name {
                Variant::String(s) => s.clone(),
                _ => continue,
            };
            process_names.insert(pid_str.clone(), name_str);

            if let Some(command_line) = process.get("CommandLine") {
                if let Variant::String(cmd) = command_line {
                    process_command_lines.insert(pid_str, cmd.clone());
                }
            }
        }
    }

    unsafe {
        let mut size: u32 = 0;
        let ret = GetExtendedUdpTable(
            None,
            &mut size as *mut u32,
            false,
            AF_INET.0 as u32,
            UDP_TABLE_OWNER_MODULE,
            0,
        );

        if ret != ERROR_INSUFFICIENT_BUFFER.0 {
            panic!("GetExtendedUdpTable failed: {}", ret);
        }

        let mut buffer = vec![0u8; size as usize];

        let ret = GetExtendedUdpTable(
            Some(buffer.as_mut_ptr() as *mut _),
            &mut size as *mut u32,
            false,
            AF_INET.0 as u32,
            UDP_TABLE_OWNER_MODULE,
            0,
        );

        if ret != 0 {
            panic!("GetExtendedUdpTable failed: {}", ret);
        }

        let table_ptr = buffer.as_ptr() as *const MIB_UDPTABLE_OWNER_MODULE;
        let table = &*table_ptr;
        let num_entries = table.dwNumEntries;

        let rows_ptr = &table.table as *const MIB_UDPROW_OWNER_MODULE;

        for i in 0..num_entries {
            let row = &*rows_ptr.add(i as usize);

            let pid = row.dwOwningPid;
            let local_port = u16::from_be((row.dwLocalPort & 0xFFFF) as u16);
            let local_addr = std::net::Ipv4Addr::from(row.dwLocalAddr.to_be());
            let create_timestamp = filetime_to_systemtime(row.liCreateTimestamp);
            let ts_str = fmt_local(create_timestamp);
            let spesific_port_bind_flags = row.Anonymous.Anonymous._bitfield;
            let pid_str = pid.to_string();
            let process_name = process_names
                .get(&pid_str)
                .map(|s| s.as_str())
                .unwrap_or("Unknown");
            let command_line = process_command_lines
                .get(&pid_str)
                .map(|s| s.as_str())
                .unwrap_or("N/A");
            let service_tag = row.OwningModuleInfo[0];
            let service_name =
                get_service_name_from_tag(pid, service_tag as u32).unwrap_or("".to_string());

            println!(
                "[{}] PID: {}\n    Local Address: {}:{}\n    Created Timestamp: {}\n    SpecificPortBindFlags: {}\n    Process Name: {}\n    Process Args: {}\n    Service Name: {}\n",
                "+".bright_red(),
                pid,
                local_addr,
                local_port,
                ts_str,
                spesific_port_bind_flags,
                process_name,
                command_line,
                service_name,
            );
        }
    }

    Ok(())
}
