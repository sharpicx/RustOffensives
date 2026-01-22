use wmi::{Variant, WMIConnection};

use crate::helper::log_success;
use crate::lib::fileutil::is_dotnet_assembly;
use crate::lib::ntdll::ProtectionValue;
use crate::lib::putil::{get_parent_pid, get_process_name, get_process_protection_info};
use crate::lib::wmiutil::{get_process_owner, wmi_query};

fn v_u32(v: &Variant) -> Option<u32> {
    match v {
        Variant::UI4(x) => Some(*x),
        Variant::I4(x) => Some(*x as u32),
        _ => None,
    }
}

fn v_string(v: &Variant) -> Option<String> {
    match v {
        Variant::String(s) => Some(s.clone()),
        _ => None,
    }
}

fn classify_process(exe_name: &str, parent_basename: &str) -> (bool, bool) {
    let exe = exe_name.to_lowercase();
    let parent = parent_basename.to_lowercase();

    let is_likely_service = matches!(
        exe.as_str(),
        "svchost.exe" | "services.exe" | "taskhost.exe"
    ) || matches!(
        parent.as_str(),
        "services.exe" | "svchost.exe" | "taskhost.exe"
    );

    let is_likely_user_process = !is_likely_service;

    (is_likely_service, is_likely_user_process)
}

fn normalize_cmd(s: &str) -> String {
    let s = s
        .strip_prefix(r"\??\")
        .or_else(|| s.strip_prefix(r"\\?\"))
        .unwrap_or(s);

    let s = s.replace(r#"\""#, r#"""#);
    let s = s.replace(r"\\", r"\");
    s
}

pub fn all_procs_print() -> Result<(), Box<dyn std::error::Error>> {
    let wmi_con = WMIConnection::with_namespace_path("ROOT\\CIMV2")?;
    let rows = wmi_query(
        "SELECT ProcessId, ParentProcessId, Name, ExecutablePath, CommandLine FROM Win32_Process",
    )?;
    let mut pid_to_name: std::collections::HashMap<u32, String> = std::collections::HashMap::new();
    for row in &rows {
        if let (Some(pid), Some(name)) = (
            row.get("ProcessId").and_then(v_u32),
            row.get("Name").and_then(v_string),
        ) {
            pid_to_name.insert(pid, name);
        }
    }
    for row in rows {
        let pid = row.get("ProcessId").and_then(v_u32);
        let name = row.get("Name").and_then(v_string);
        let (pid, name) = match (pid, name) {
            (Some(p), Some(n)) => (p, n),
            _ => continue,
        };

        let parent_pid_wmi = row.get("ParentProcessId").and_then(v_u32).unwrap_or(0);
        let parent_pid = if Some(parent_pid_wmi) == get_parent_pid(pid) {
            parent_pid_wmi
        } else {
            0
        };
        let parent_path = get_process_name(parent_pid).unwrap_or("N/A".to_string());
        let parent_basename = if parent_path != "N/A" {
            std::path::Path::new(&parent_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown")
        } else {
            "Unknown"
        };
        let path = row.get("ExecutablePath").and_then(v_string);
        let norm_path = path
            .as_deref()
            .map(normalize_cmd)
            .unwrap_or_else(|| "N/A".to_string());
        let command_line = row.get("CommandLine").and_then(v_string);
        let norm_cmd = command_line
            .as_deref()
            .map(normalize_cmd)
            .unwrap_or_else(|| "N/A".to_string());
        let exe_name = path
            .as_deref()
            .and_then(|p| std::path::Path::new(p).file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown");
        let is_dotnet = path.as_deref().map(is_dotnet_assembly).unwrap_or(false);
        let protection = if pid == 0 {
            None
        } else {
            get_process_protection_info(pid)
        };
        let protection_str = match protection {
            Some(v) => ProtectionValue(v).to_string(),
            None => "N/A".to_string(),
        };
        let (is_likely_service, is_likely_user_process) =
            classify_process(exe_name, parent_basename);
        let owner_self = get_process_owner(&wmi_con, pid).unwrap_or_else(|| "Unknown".to_string());
        let owner_parent =
            get_process_owner(&wmi_con, parent_pid).unwrap_or_else(|| "Unknown".to_string());

        log_success(&format!(
            "PID                           : {}\n    Parent PID                    : {} ({})\n    NAME                          : {}\n    OWNER (SELF)                  : {}\n    OWNER (PARENT)                : {}\n    DOTNET                        : {}\n    PPL (Protected Process Light) : {}\n    Is Likely Service             : {}\n    Is Likely User Process        : {}\n    PATH                          : {}\n    CMD                           : {}",
            pid,
            parent_pid,
            parent_path,
            name,
            owner_self,
            owner_parent,
            is_dotnet,
            protection_str,
            is_likely_service,
            is_likely_user_process,
            norm_path,
            norm_cmd,
        ));

        println!();
    }

    Ok(())
}
