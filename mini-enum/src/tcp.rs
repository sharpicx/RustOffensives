use std::collections::HashMap;
use widestring::U16CString;
use windows_alt::Win32::Foundation::ERROR_INSUFFICIENT_BUFFER;
use windows_alt::Win32::NetworkManagement::IpHelper::{
    GetExtendedTcpTable, MIB_TCPROW_OWNER_MODULE, MIB_TCPTABLE_OWNER_MODULE,
    TCP_TABLE_OWNER_MODULE_ALL,
};
use windows_alt::Win32::Networking::WinSock::AF_INET;
use windows_alt::core::PCWSTR;
use wmi::Variant;

use crate::helper::log_success;
use crate::lib::advapi::{I_QueryTagInformation, ScServiceTagQuery, TagInfoLevel};
use crate::lib::kernel32::LocalFree;
use crate::lib::tcputil::TcpState;
use crate::lib::wmiutil::wmi_query;

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

pub fn tcp_connections() -> Result<(), Box<dyn std::error::Error>> {
    let mut process_names: HashMap<String, String> = HashMap::new();
    let mut process_command_lines: HashMap<String, String> = HashMap::new();
    let rows: Vec<HashMap<String, Variant>> = wmi_query("SELECT * FROM Win32_Process")?;
    for process in rows {
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
        let ret = GetExtendedTcpTable(
            None,
            &mut size as *mut u32,
            false,
            AF_INET.0 as u32,
            TCP_TABLE_OWNER_MODULE_ALL,
            0,
        );

        if ret != ERROR_INSUFFICIENT_BUFFER.0 {
            panic!("GetExtendedTcpTable failed: {}", ret);
        }

        let mut buffer = vec![0u8; size as usize];

        let ret = GetExtendedTcpTable(
            Some(buffer.as_mut_ptr() as *mut _),
            &mut size as *mut u32,
            false,
            AF_INET.0 as u32,
            TCP_TABLE_OWNER_MODULE_ALL,
            0,
        );

        if ret != 0 {
            panic!("GetExtendedTcpTable failed: {}", ret);
        }

        let table_ptr = buffer.as_ptr() as *const MIB_TCPTABLE_OWNER_MODULE;
        let table = &*table_ptr;
        let num_entries = table.dwNumEntries;

        let rows_ptr = &table.table as *const MIB_TCPROW_OWNER_MODULE;

        for i in 0..num_entries {
            let row = &*rows_ptr.add(i as usize);

            let pid = row.dwOwningPid;
            let local_port = u16::from_be((row.dwLocalPort & 0xFFFF) as u16);
            let remote_port = u16::from_be((row.dwRemotePort & 0xFFFF) as u16);
            let local_addr = std::net::Ipv4Addr::from(row.dwLocalAddr.to_be());
            let remote_addr = std::net::Ipv4Addr::from(row.dwRemoteAddr.to_be());
            let state: TcpState = TcpState::from(row.dwState);
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
                get_service_name_from_tag(pid, service_tag as u32).unwrap_or("N/A".to_string());

            log_success(&format!(
                "PID: {}\n    Local Address: {}:{}\n    Remote Address: {}:{}\n    State: {}\n    Process Name: {}\n    Process Args: {}\n    Service Name: {}\n",
                pid,
                local_addr,
                local_port,
                remote_addr,
                remote_port,
                state,
                process_name,
                command_line,
                service_name,
            ));
        }
    }

    Ok(())
}
