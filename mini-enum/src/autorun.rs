use crate::helper::log_success;
use winreg::RegKey;
use winreg::RegValue;
use winreg::enums::HKEY_LOCAL_MACHINE;
use winreg::enums::REG_EXPAND_SZ;
use winreg::enums::REG_SZ;

pub fn get_autorun() {
    let autorun_locations = [
        "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run",
        "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\RunOnce",
        "SOFTWARE\\Wow6432Node\\Microsoft\\Windows\\CurrentVersion\\Run",
        "SOFTWARE\\Wow6432Node\\Microsoft\\Windows\\CurrentVersion\\RunOnce",
        "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\RunService",
        "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\RunOnceService",
        "SOFTWARE\\Wow6432Node\\Microsoft\\Windows\\CurrentVersion\\RunService",
        "SOFTWARE\\Wow6432Node\\Microsoft\\Windows\\CurrentVersion\\RunOnceService",
    ];

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

    for location in autorun_locations.iter() {
        if let Ok(key) = hklm.open_subkey(location) {
            let values: Vec<String> = key
                .enum_values()
                .filter_map(Result::ok)
                .filter_map(|(_, val): (String, RegValue)| match val.vtype {
                    REG_SZ | REG_EXPAND_SZ => Some(String::from_utf16_lossy(
                        &val.bytes
                            .chunks(2)
                            .map(|c| u16::from_le_bytes([c[0], c[1]]))
                            .collect::<Vec<u16>>(),
                    )),
                    _ => None,
                })
                .collect();

            if !values.is_empty() {
                log_success(&format!("HKLM:\\{}", location));
                for val in values {
                    log_success(&format!("  {}", val));
                }
            }
        }
    }
    println!();
}
