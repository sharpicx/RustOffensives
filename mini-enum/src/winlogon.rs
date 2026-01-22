use crate::helper::log_success;
use std::io;
use winreg::RegKey;
use winreg::enums::*;

#[derive(Debug)]
struct WindowsAutoLogonDTO {
    default_domain: Option<String>,
    default_user: Option<String>,
    default_password: Option<String>,
    alt_default_domain: Option<String>,
    alt_default_user: Option<String>,
    alt_default_password: Option<String>,
}

pub fn enum_autologon() -> io::Result<()> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let winlogon_path = "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Winlogon";

    if let Ok(winlogon) = hklm.open_subkey(winlogon_path) {
        let dto = WindowsAutoLogonDTO {
            default_domain: winlogon.get_value("DefaultDomainName").ok(),
            default_user: winlogon.get_value("DefaultUserName").ok(),
            default_password: winlogon.get_value("DefaultPassword").ok(),
            alt_default_domain: winlogon.get_value("AltDefaultDomainName").ok(),
            alt_default_user: winlogon.get_value("AltDefaultUserName").ok(),
            alt_default_password: winlogon.get_value("AltDefaultPassword").ok(),
        };

        log_success("Windows Auto-Logon:");
        log_success(&format!(
            "Default Domain: {:?}",
            dto.default_domain.as_deref().unwrap_or("")
        ));
        log_success(&format!(
            "Default User: {:?}",
            dto.default_user.as_deref().unwrap_or("")
        ));
        log_success(&format!(
            "Default Password: {:?}",
            dto.default_password.as_deref().unwrap_or("")
        ));
        log_success(&format!(
            "Alt Default Domain: {:?}",
            dto.alt_default_domain.as_deref().unwrap_or("")
        ));
        log_success(&format!(
            "Alt Default User: {:?}",
            dto.alt_default_user.as_deref().unwrap_or("")
        ));
        log_success(&format!(
            "Alt Default Password: {:?}",
            dto.alt_default_password
        ));
    }

    println!();
    Ok(())
}
