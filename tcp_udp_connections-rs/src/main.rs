use colored::*;
use std::collections::HashMap;
use widestring::U16CString;
use windows::Win32::Foundation::ERROR_INSUFFICIENT_BUFFER;
use windows::Win32::NetworkManagement::IpHelper::{
    GetExtendedTcpTable, MIB_TCPROW_OWNER_MODULE, MIB_TCPTABLE_OWNER_MODULE,
    TCP_TABLE_OWNER_MODULE_ALL,
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

#[derive(Debug)]
enum TcpState {
    Closed = 1,
    Listen = 2,
    SynSent = 3,
    SynReceived = 4,
    Established = 5,
    FinWait1 = 6,
    FinWait2 = 7,
    CloseWait = 8,
    Closing = 9,
    LastAck = 10,
    TimeWait = 11,
    DeleteTcb = 12,
}

impl From<u32> for TcpState {
    fn from(state: u32) -> Self {
        match state {
            1 => TcpState::Closed,
            2 => TcpState::Listen,
            3 => TcpState::SynSent,
            4 => TcpState::SynReceived,
            5 => TcpState::Established,
            6 => TcpState::FinWait1,
            7 => TcpState::FinWait2,
            8 => TcpState::CloseWait,
            9 => TcpState::Closing,
            10 => TcpState::LastAck,
            11 => TcpState::TimeWait,
            12 => TcpState::DeleteTcb,
            _ => TcpState::Closed,
        }
    }
}

impl std::fmt::Display for TcpState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TcpState::Closed => "CLOSED",
            TcpState::Listen => "LISTEN",
            TcpState::SynSent => "SYNSENT",
            TcpState::SynReceived => "SYNRECEIVED",
            TcpState::Established => "ESTABLISHED",
            TcpState::FinWait1 => "FINWAIT1",
            TcpState::FinWait2 => "FINWAIT2",
            TcpState::CloseWait => "CLOSEWAIT",
            TcpState::Closing => "CLOSING",
            TcpState::LastAck => "LASTACK",
            TcpState::TimeWait => "TIMEWAIT",
            TcpState::DeleteTcb => "DELETETCB",
        };
        write!(f, "{}", s)
    }
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
                get_service_name_from_tag(pid, service_tag as u32).unwrap_or("".to_string());

            println!(
                "[{}] PID: {}\n    local_address: {}:{}\n    remote_address: {}:{}\n    state: {}\n    process_name: {}\n    command_line: {}\n    service_name: {}\n",
                "+".bright_red(),
                pid,
                local_addr,
                local_port,
                remote_addr,
                remote_port,
                state,
                process_name,
                command_line,
                service_name,
            );
        }
    }

    Ok(())
}
