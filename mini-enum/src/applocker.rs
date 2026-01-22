use crate::lib::wmiutil::wmi_query;
use winreg_new::RegKey;
use winreg_new::enums::*;
use wmi::Variant;

pub fn enum_applocker_settings() -> Result<(), Box<dyn std::error::Error>> {
    let query = "SELECT Name, State FROM Win32_Service WHERE Name = 'AppIDSvc'";
    let results = wmi_query(query)?;
    let mut app_id_svc_state = "Service not found".to_string();

    for row in results {
        if let Some(Variant::String(state)) = row.get("State") {
            app_id_svc_state = state.clone();
        }
    }

    println!("AppIDSvc State: {}", app_id_svc_state);

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let srp_base = hklm.open_subkey("Software\\Policies\\Microsoft\\Windows\\SrpV2");

    match srp_base {
        Ok(base) => {
            let mut any_configured = false;
            for key_name in base.enum_keys().flatten() {
                any_configured = true;
                let key_path = format!(
                    "Software\\Policies\\Microsoft\\Windows\\SrpV2\\{}",
                    key_name
                );
                let key = hklm.open_subkey(&key_path)?;

                let enforcement_mode: Option<u32> = key.get_value("EnforcementMode").ok();
                let enforcement_str = match enforcement_mode {
                    None => "not configured".to_string(),
                    Some(0) => "Audit Mode".to_string(),
                    Some(1) => "Enforce Mode".to_string(),
                    Some(v) => format!("Unknown value {}", v),
                };

                println!("Key: {}, EnforcementMode: {}", key_name, enforcement_str);

                let mut rules = Vec::new();
                for id in key.enum_keys().flatten() {
                    let rule_path = format!(
                        "Software\\Policies\\Microsoft\\Windows\\SrpV2\\{}\\{}",
                        key_name, id
                    );
                    if let Some(rule_value) = hklm
                        .open_subkey(&rule_path)
                        .ok()
                        .and_then(|rk| rk.get_value::<String, _>("Value").ok())
                    {
                        rules.push(rule_value);
                    }
                }
                if rules.is_empty() {
                    println!("No rules configured for this key");
                } else {
                    for r in rules {
                        println!("Rule: {}", r);
                    }
                }
            }
            if !any_configured {
                println!("AppLocker not configured");
            }
        }
        Err(_) => {
            println!("AppLocker not configured");
        }
    }

    Ok(())
}
