use colored::*;
use std::ptr;
use windows_old::Win32::Foundation::*;
use windows_old::Win32::Security::*;
use windows_old::Win32::System::Threading::*;

pub struct TokenPrivilegeDTO {
    pub privilege: String,
    pub attributes: u32,
}

// ported from https://github.com/GhostPack/Seatbelt/blob/master/Seatbelt/Commands/Windows/TokenPrivilegesCommand.cs#L36
fn get_privs() -> windows_old::core::Result<Vec<TokenPrivilegeDTO>> {
    unsafe {
        let mut token_handle = HANDLE::default();

        if !OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token_handle).as_bool() {
            return Err(windows_old::core::Error::from_win32());
        }
        let mut return_len = 0;

        GetTokenInformation(token_handle, TokenPrivileges, None, 0, &mut return_len);

        let mut buffer = vec![0u8; return_len as usize];

        if !GetTokenInformation(
            token_handle,
            TokenPrivileges,
            Some(buffer.as_mut_ptr() as *mut _),
            return_len,
            &mut return_len,
        )
        .as_bool()
        {
            CloseHandle(token_handle);
            return Err(windows_old::core::Error::from_win32());
        }

        let token_privs = &*(buffer.as_ptr() as *const TOKEN_PRIVILEGES);
        let mut results = Vec::new();

        let privileges: &[LUID_AND_ATTRIBUTES] = std::slice::from_raw_parts(
            token_privs.Privileges.as_ptr(),
            token_privs.PrivilegeCount as usize,
        );

        for laa in privileges {
            let mut name_len: u32 = 0;
            LookupPrivilegeNameW(
                None,
                &laa.Luid,
                windows_old::core::PWSTR(ptr::null_mut()),
                &mut name_len,
            );

            if name_len > 0 {
                let mut name_buf: Vec<u16> = vec![0; (name_len + 1) as usize];
                if LookupPrivilegeNameW(
                    None,
                    &laa.Luid,
                    windows_old::core::PWSTR(name_buf.as_mut_ptr()),
                    &mut name_len,
                )
                .as_bool()
                {
                    let privilege_name = String::from_utf16_lossy(&name_buf[..name_len as usize]);
                    results.push(TokenPrivilegeDTO {
                        privilege: privilege_name,
                        attributes: laa.Attributes.0,
                    });
                }
            }
        }

        CloseHandle(token_handle);
        Ok(results)
    }
}

pub fn enum_privs() -> Result<(), windows_old::core::Error> {
    let privs = get_privs()?;

    println!("{}", "[+] Current User Privileges:".green());

    if !privs.is_empty() {
        for p in &privs {
            let status = match p.attributes {
                0 => "Enabled by default",
                1 => "SE_PRIVILEGE_ENABLED_BY_DEFAULT",
                2 => "Enabled by admin via token",
                3 => "SE_PRIVILEGE_ENABLED | SE_PRIVILEGE_ENABLED_BY_DEFAULT = enabled + present",
                _ => "Unknown",
            };

            println!("     {}", p.privilege.cyan());
            println!(
                "         - {}: {} ({})",
                "Attributes".yellow(),
                p.attributes,
                status.bright_yellow(),
            );
        }
    }

    let mut found = false;
    let dangerous: Vec<(&str, &[&str])> = vec![
        (
            "SeAssignPrimaryTokenPrivilege",
            &[
                "It would allow a user to impersonate tokens and privesc to nt system using tools such as potato.exe, rottenpotato.exe and juicypotato.exe",
                "https://twitter.com/Defte_",
            ],
        ),
        (
            "SeCreateTokenPrivilege",
            &[
                "Create arbitrary token including local admin rights with NtCreateToken.",
                "https://github.com/daem0nc0re/PrivFu/blob/main/PrivilegedOperations/SeCreateTokenPrivilegePoC/SeCreateTokenPrivilegePoC.cs",
            ],
        ),
        (
            "SeDebugPrivilege",
            &[
                "https://github.com/xct/SeDebugAbuse",
                "https://github.com/joaoviictorti/SeDebugAbuse-rs",
            ],
        ),
        (
            "SeTakeOwnershipPrivilege",
            &[
                "Attack may be detected by some AV software.",
                r#"Alternative method relies on replacing service binaries stored in "Program Files" using the same privilege."#,
                "If you have SeTakeOwnershipPrivilege, you can change owner of any files and registries to caller.",
                "This PoC tries to change owner of a privileged registry key to caller of this PoC.",
                "https://github.com/daem0nc0re/PrivFu/blob/main/PrivilegedOperations/SeTakeOwnershipPrivilegePoC/SeTakeOwnershipPrivilegePoC.cs",
            ],
        ),
        (
            "SeTcbPrivilege",
            &[
                "If you have SeTcbPrivilege, you can perform S4U Logon.",
                r#"This PoC tries to perform S4U Logon and add "Builtin\Backup Operators\" to current token group."#,
                "https://github.com/daem0nc0re/PrivFu/blob/main/PrivilegedOperations/SeTcbPrivilegePoC/SeTcbPrivilegePoC.cs",
            ],
        ),
        (
            "SeBackupPrivilege",
            &[
                "https://github.com/k4sth4/SeBackupPrivilege",
                "https://github.com/giuliano108/SeBackupPrivilege",
                "https://github.com/nickvourd/Windows-Local-Privilege-Escalation-Cookbook/blob/master/Notes/SeBackupPrivilege.md",
                "https://github.com/daem0nc0re/PrivFu/tree/main/PrivilegedOperations/SeBackupPrivilegePoC",
            ],
        ),
        (
            "SeRestorePrivilege",
            &["https://github.com/xct/SeRestoreAbuse"],
        ),
        (
            "SeLoadDriverPrivilege",
            &[
                "https://www.greyhathacker.net/?p=1025",
                "https://cve.mitre.org/cgi-bin/cvename.cgi?name=CVE-2018-15732",
            ],
        ),
        (
            "SeManageVolumePrivilege",
            &["https://github.com/xct/SeManageVolumeAbuse"],
        ),
        (
            "SeImpersonatePrivilege",
            &[
                "Tools from the Potato family (potato.exe, RottenPotato, RottenPotatoNG, Juicy Potato, SweetPotato, RemotePotato0), RogueWinRM, PrintSpoofer, etc.",
            ],
        ),
        (
            "SeRelabelPrivilege",
            &[
                "https://github.com/decoder-it/RelabelAbuse",
                "https://www.tiraniddo.dev/2021/06/the-much-misunderstood.html",
            ],
        ),
        (
            "SeSecurityPrivilege",
            &[
                "If you have SeSecurityPrivilege, you can read security events.",
                "This PoC tries to read the latest security event.",
                "https://github.com/daem0nc0re/PrivFu/blob/main/PrivilegedOperations/SeSecurityPrivilegePoC/SeSecurityPrivilegePoC.cs",
            ],
        ),
        (
            "SeEnableDelegationPrivilege",
            &[
                "Unconstrained Delegation? Check it out",
                "https://www.netspi.com/blog/technical-blog/network-penetration-testing/machineaccountquota-is-useful-sometimes/",
                "https://en.hackndo.com/constrained-unconstrained-delegation/",
            ],
        ),
    ];
    for p in &privs {
        for (name, refs) in &dangerous {
            if p.privilege == *name && p.attributes != 0 {
                if !found {
                    println!("\n{}", "[+] Dangerous Privileges Detected:".green());
                    found = true;
                }
                println!("     - {}", name.red());
                println!("         - {}:", "References".bright_blue());
                for r in *refs {
                    println!("             - {}", r);
                }
            }
        }
    }
    println!();
    Ok(())
}
