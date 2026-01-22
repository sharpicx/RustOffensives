use colored::*;
use regex::Regex;
use std::collections::HashMap;
use std::ffi::c_void;
use std::process::Command;
use std::ptr;
use winapi::shared::lmcons::MAX_PREFERRED_LENGTH;
use winapi::um::lmaccess::FILTER_NORMAL_ACCOUNT;
use winapi::um::lmaccess::LOCALGROUP_INFO_1;
use winapi::um::lmaccess::NetLocalGroupEnum;
use winapi::um::lmaccess::NetUserEnum;
use winapi::um::lmaccess::USER_INFO_2;
use winapi::um::lmapibuf::NetApiBufferFree;

pub fn enum_desc_users() -> windows_old::core::Result<()> {
    unsafe {
        let mut buf: *mut c_void = ptr::null_mut();
        let mut entries_read: u32 = 0;
        let mut total_entries: u32 = 0;
        let mut resume_handle: u32 = 0;

        let status = NetUserEnum(
            ptr::null(),
            2,
            FILTER_NORMAL_ACCOUNT,
            &mut buf as *mut *mut c_void as *mut *mut u8,
            MAX_PREFERRED_LENGTH,
            &mut entries_read,
            &mut total_entries,
            &mut resume_handle,
        );

        if status != 0 {
            return Err(windows_old::core::Error::from_win32());
        }

        if buf.is_null() || entries_read == 0 {
            println!("[*] No users found.");
            return Ok(());
        }

        let users: &[USER_INFO_2] =
            std::slice::from_raw_parts(buf as *const USER_INFO_2, entries_read as usize);

        println!("{}", "[+] User Comments & Full Name:".green());

        for u in users {
            let name = if !u.usri2_name.is_null() {
                let len = (0..).take_while(|&i| *u.usri2_name.add(i) != 0).count();
                String::from_utf16_lossy(std::slice::from_raw_parts(u.usri2_name, len))
            } else {
                String::new()
            };

            let full_name = if !u.usri2_full_name.is_null() {
                let len = (0..)
                    .take_while(|&i| *u.usri2_full_name.add(i) != 0)
                    .count();
                String::from_utf16_lossy(std::slice::from_raw_parts(u.usri2_full_name, len))
            } else {
                String::new()
            };

            let comment = if !u.usri2_comment.is_null() {
                let len = (0..).take_while(|&i| *u.usri2_comment.add(i) != 0).count();
                String::from_utf16_lossy(std::slice::from_raw_parts(u.usri2_comment, len))
            } else {
                String::new()
            };

            println!("     [{}]", name.yellow());
            println!(
                "         - [{}]: {}",
                "Full Name".bright_blue(),
                full_name.white()
            );
            println!(
                "         - [{}]: {}",
                "Comments".bright_blue(),
                comment.white()
            );
            println!();
        }

        NetApiBufferFree(buf as *mut _);
    }

    Ok(())
}

pub fn enum_desc_groups() -> windows_old::core::Result<()> {
    unsafe {
        let mut buf: *mut c_void = std::ptr::null_mut();
        let mut entries_read: u32 = 0;
        let mut total_entries: u32 = 0;
        let mut resume_handle: usize = 0;

        let status = NetLocalGroupEnum(
            std::ptr::null(),
            1,
            &mut buf as *mut *mut c_void as *mut *mut u8,
            MAX_PREFERRED_LENGTH,
            &mut entries_read,
            &mut total_entries,
            &mut resume_handle,
        );

        if status != 0 {
            return Err(windows_old::core::Error::from_win32());
        }

        if buf.is_null() || entries_read == 0 {
            println!("[*] No local groups found.");
            return Ok(());
        }

        let groups: &[LOCALGROUP_INFO_1] =
            std::slice::from_raw_parts(buf as *const LOCALGROUP_INFO_1, entries_read as usize);

        println!("{}", "[+] Local Group Comments:".green());

        for g in groups {
            let name = if !g.lgrpi1_name.is_null() {
                let len = (0..).take_while(|&i| *g.lgrpi1_name.add(i) != 0).count();
                String::from_utf16_lossy(std::slice::from_raw_parts(g.lgrpi1_name, len))
            } else {
                String::new()
            };

            let comment = if !g.lgrpi1_comment.is_null() {
                let len = (0..).take_while(|&i| *g.lgrpi1_comment.add(i) != 0).count();
                String::from_utf16_lossy(std::slice::from_raw_parts(g.lgrpi1_comment, len))
            } else {
                String::new()
            };

            println!("     [{}]", name.yellow());
            println!(
                "         - [{}]: {}",
                "Comment".bright_blue(),
                comment.white()
            );
            println!();
        }

        NetApiBufferFree(buf as *mut _);
    }

    Ok(())
}

fn run_cmd(cmd: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| format!("failed to spawn {}: {}", cmd, e))?;
    let out = String::from_utf8_lossy(&output.stdout).to_string();
    let err = String::from_utf8_lossy(&output.stderr).to_string();
    if !output.status.success() && out.trim().is_empty() {
        return Err(format!("{} returned non-zero: {}", cmd, err));
    }
    Ok(out)
}

fn build_group_db() -> HashMap<String, &'static str> {
    let mut m = HashMap::new();
    macro_rules! add {
        ($k:expr, $v:expr) => {
            m.insert($k.to_lowercase(), $v);
        };
    }

    add!(
        "BUILTIN\\Administrators",
        "Full admin on the local machine. Members can perform any action locally, install software, manage users, and elevate privileges."
    );
    add!(
        "Administrators",
        "Full admin on the local machine. Equivalent to BUILTIN\\Administrators in many outputs."
    );
    add!(
        "BUILTIN\\Users",
        "Standard non-admin users. Can run applications and read/write allowed profiles; limited ability to change system settings."
    );
    add!("Users", "Standard non-admin users group.");
    add!(
        "BUILTIN\\Guests",
        "Minimal privileges; typically restricted account for temporary access."
    );
    add!("Guests", "Minimal-privilege group.");
    add!(
        "BUILTIN\\Power Users",
        "Legacy elevated group (pre-Vista). Some additional rights but not full admin."
    );
    add!(
        "BUILTIN\\Backup Operators",
        "Can back up and restore files regardless of ACLs; useful for data exfiltration or backup tasks."
    );
    add!(
        "BUILTIN\\Replicator",
        "Used for file replication service; rare on modern systems."
    );
    add!(
        "NT AUTHORITY\\SYSTEM",
        "The SYSTEM account (highest local privilege). Processes running as SYSTEM can do anything on machine."
    );
    add!(
        "NT AUTHORITY\\LOCAL SERVICE",
        "Service account with limited local privileges and anonymous network credentials."
    );
    add!(
        "NT AUTHORITY\\NETWORK SERVICE",
        "Service account with limited local privileges but presents machine credentials to network."
    );
    add!(
        "NT AUTHORITY\\Authenticated Users",
        "All users who have authenticated (not guests/anonymous). Commonly used in ACLs."
    );
    add!(
        "NT AUTHORITY\\Everyone",
        "All principals including anonymous in some contexts; broad access."
    );
    add!(
        "NT AUTHORITY\\INTERACTIVE",
        "Users who are logged on interactively (console or RDP)."
    );
    add!(
        "NT AUTHORITY\\REMOTE INTERACTIVE LOGON",
        "Users logged on via RDP / remote interactive sessions."
    );
    add!(
        "Mandatory Label\\High Mandatory Level",
        "Integrity level used by Windows Mandatory Integrity Control. High usually means elevated (Admin) processes."
    );
    add!(
        "Mandatory Label\\Medium Mandatory Level",
        "Default integrity for standard processes."
    );
    add!(
        "Domain Users",
        "In AD: default group for domain user accounts. Basic domain-level account membership."
    );
    add!(
        "Domain Computers",
        "In AD: group containing computer objects in domain."
    );
    add!(
        "Domain Admins",
        "Domain-level admins with full control across the domain (powerful, often high-value)."
    );
    add!(
        "Domain Controllers",
        "Group containing domain controller machines."
    );
    add!(
        "Enterprise Admins",
        "Forest-level admins; extremely powerful across the AD forest."
    );
    add!(
        "Schema Admins",
        "Can modify Active Directory schema; rarely used day-to-day."
    );
    add!(
        "Account Operators",
        "Can create/modify non-admin accounts in domain; limited compared to Domain Admins."
    );
    add!(
        "Server Operators",
        "Can manage servers in domain (start/stop services, share management)."
    );
    add!(
        "DnsAdmins",
        "Administrate DNS servers; can modify DNS records which may enable redirection."
    );
    add!(
        "Group Policy Creator Owners",
        "Can create new GPOs (Group Policy Objects)."
    );
    add!(
        "BUILTIN\\Remote Desktop Users",
        "Allows RDP logon to the machine."
    );
    add!("Remote Desktop Users", "Allows RDP logon.");
    add!(
        "BUILTIN\\IIS_IUSRS",
        "IIS worker-process group; used for web-related permissions."
    );
    add!(
        "BUILTIN\\Hyper-V Administrators",
        "Manage Hyper-V VMs and virtualization host."
    );
    add!(
        "BUILTIN\\Event Log Readers",
        "Can read event logs; useful for reconnaissance."
    );
    add!(
        "BUILTIN\\Performance Monitor Users",
        "Can read performance counters."
    );
    add!(
        "BUILTIN\\Performance Log Users",
        "Can manage performance logs and alerts."
    );
    add!(
        "NT AUTHORITY\\Local Account",
        "Indicates a local account (not domain)."
    );
    add!(
        "NT AUTHORITY\\Local Account and Member of Administrators Group",
        "Local account that is also in administrators group (elevated local account)."
    );

    m
}

fn parse_whoami_groups(out: &str) -> Vec<String> {
    let mut res = Vec::new();
    let mut started = false;

    for line in out.lines() {
        let l = line.trim_end();

        if l.starts_with("-----") || l.starts_with("=") {
            started = true;
            continue;
        }
        if !started {
            continue;
        }
        if l.is_empty() {
            continue;
        }

        if let Some(pos) = l.find("  ") {
            let name = l[..pos].trim();
            if !name.is_empty() && name != "Group Name" {
                res.push(name.to_string());
            }
        }
    }

    res
}

fn parse_net_user_list(out: &str) -> Vec<String> {
    let mut users = Vec::new();
    let start_re = Regex::new(r"^---+").unwrap();
    let mut started = false;
    for line in out.lines() {
        if start_re.is_match(line) {
            started = true;
            continue;
        }
        if !started {
            continue;
        }
        if line
            .to_lowercase()
            .contains("command completed successfully")
        {
            break;
        }
        for tok in line.split_whitespace() {
            users.push(tok.to_string());
        }
    }
    if users.is_empty() {
        for line in out.lines() {
            if line.trim().is_empty() {
                continue;
            }
            if line.contains("User accounts for")
                || line
                    .to_lowercase()
                    .contains("command completed successfully")
            {
                continue;
            }
            for tok in line.split_whitespace() {
                if !tok.contains('\\') {
                    users.push(tok.to_string());
                }
            }
        }
    }
    users
}

fn parse_net_user_groups(out: &str) -> Vec<String> {
    let mut groups = Vec::new();

    let mut in_local = false;
    let mut in_global = false;

    const DATA_COL: usize = 26;

    for line in out.lines() {
        if line.contains("The command completed successfully") {
            break;
        }

        if line.trim().is_empty() {
            continue;
        }

        let trimmed = line.trim_start().to_lowercase();

        if trimmed.starts_with("local group") {
            in_local = true;
            in_global = false;

            if line.len() > DATA_COL {
                let data = &line[DATA_COL..];
                groups.extend(split_groups(data));
            }
            continue;
        }

        if trimmed.starts_with("global group") {
            in_local = false;
            in_global = true;

            if line.len() > DATA_COL {
                let data = &line[DATA_COL..];
                groups.extend(split_groups(data));
            }
            continue;
        }

        if (in_local || in_global) && line.starts_with(' ') {
            if line.len() > DATA_COL {
                let data = &line[DATA_COL..];
                groups.extend(split_groups(data));
            }
            continue;
        }

        in_local = false;
        in_global = false;
    }

    groups
}

fn split_groups(s: &str) -> Vec<String> {
    let mut result = Vec::new();

    for part in s.split('*') {
        let g = part.trim();
        if g.is_empty() {
            continue;
        }
        result.push(g.to_string());
    }

    result
}

fn print_group_with_desc(name: &str, db: &HashMap<String, &str>) {
    let key = name.to_lowercase();
    let desc = db.get(&key)
        .or_else(|| {
            if let Some(pos) = key.find('\\') {
                db.get(&key[pos+1..].to_string())
            } else { None }
        })
        .map(|s| *s)
        .unwrap_or("No builtin description. This group likely provides specific rights/ACLs; investigate local/group policy and ACLs.");
    println!("    - {}", name.bright_blue());
    println!("      - {}", desc);
}

pub fn enum_common_groups() {
    if !cfg!(target_os = "windows") {
        eprintln!("This tool is intended to run on Windows.");
        std::process::exit(1);
    }

    let db = build_group_db();
    let whoami = match run_cmd("whoami", &["/groups"]) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("failed to run whoami: {}", e);
            String::new()
        }
    };
    let current_user = match run_cmd("whoami", &[]) {
        Ok(o) => o.trim().to_string(),
        Err(_) => "Unknown".to_string(),
    };

    println!("[+] Current User: {}", current_user.green());
    println!("[+] Groups:");

    let groups = parse_whoami_groups(&whoami);
    if groups.is_empty() {
        println!("    (no groups found via whoami parsing)");
    } else {
        for g in groups.iter() {
            print_group_with_desc(g, &db);
        }
    }

    let net_user_out = match run_cmd("net", &["user"]) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("failed to run `net user`: {}", e);
            String::new()
        }
    };

    let users = parse_net_user_list(&net_user_out);
    if users.is_empty() {
        println!("\n[!] No local users parsed from `net user` output.");
        return;
    }

    println!("\n{}", "[+] Local Users and their groups:".green());
    for u in users.iter() {
        println!("  [{}] ", u.yellow());
        let out = match run_cmd("net", &["user", u]) {
            Ok(o) => o,
            Err(_) => {
                println!("    - (failed to query user details)");
                continue;
            }
        };
        let ugroups = parse_net_user_groups(&out);
        if ugroups.is_empty() {
            println!("    - (no local/global group memberships parsed)");
        } else {
            for g in ugroups.iter() {
                print_group_with_desc(g, &db);
            }
        }
    }
    println!();
}
