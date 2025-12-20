use std::collections::HashMap;
use wmi::{IWbemClassWrapper, Variant, WMIConnection};

fn wmi_cimv2(
    con: &WMIConnection,
    q: &str,
) -> Result<Vec<HashMap<String, Variant>>, Box<dyn std::error::Error>> {
    let rows: Vec<HashMap<String, Variant>> = con.raw_query(q)?;
    Ok(rows)
}

fn variant_to_string(v: Option<&Variant>) -> String {
    match v {
        Some(Variant::String(s)) => s.clone(),
        Some(Variant::UI4(v)) => v.to_string(),
        Some(Variant::I4(v)) => v.to_string(),
        _ => String::new(),
    }
}

// https://docs.rs/wmi/latest/wmi/struct.WMIConnection.html#method.exec_method
// https://github.com/GhostPack/Seatbelt/blob/master/Seatbelt/Commands/Windows/ProcessOwnersCommand.cs#L35
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let con = WMIConnection::with_namespace_path("ROOT\\CIMV2")?;
    let query = "SELECT ProcessId, Name FROM Win32_Process WHERE SessionID != 0";
    let results = wmi_cimv2(&con, query)?;
    for proc in results {
        let pid = match proc.get("ProcessId") {
            Some(Variant::UI4(v)) => *v,
            _ => continue,
        };
        let name = variant_to_string(proc.get("Name"));
        let object_path = format!("Win32_Process.Handle=\"{}\"", pid);
        let out: Option<IWbemClassWrapper> = con.exec_method(&object_path, "GetOwner", None)?;
        let out = match out {
            Some(v) => v,
            None => continue,
        };
        let user = variant_to_string(Some(&out.get_property("User")?));
        let domain = variant_to_string(Some(&out.get_property("Domain")?));
        println!(
            "PID: {} | Name: {} | Owner: {}\\{}",
            pid, name, domain, user
        );
    }

    Ok(())
}
