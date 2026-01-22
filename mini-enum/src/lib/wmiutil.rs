use std::collections::HashMap;
use wmi::{IWbemClassWrapper, Variant, WMIConnection};

pub fn get_process_owner(con: &WMIConnection, pid: u32) -> Option<String> {
    fn variant_to_string(v: Option<&Variant>) -> String {
        match v {
            Some(Variant::String(s)) => s.clone(),
            Some(Variant::UI4(v)) => v.to_string(),
            Some(Variant::I4(v)) => v.to_string(),
            _ => String::new(),
        }
    }
    let object_path = format!("Win32_Process.Handle=\"{}\"", pid);
    let out: IWbemClassWrapper = con.exec_method(&object_path, "GetOwner", None).ok()??;
    let user = variant_to_string(Some(&out.get_property("User").ok()?));
    let domain = variant_to_string(Some(&out.get_property("Domain").ok()?));
    if user.is_empty() && domain.is_empty() {
        None
    } else {
        Some(format!("{}\\{}", domain, user))
    }
}

pub fn wmi_query(q: &str) -> Result<Vec<HashMap<String, Variant>>, Box<dyn std::error::Error>> {
    let con = WMIConnection::new()?;
    let rows: Vec<HashMap<String, Variant>> = con.raw_query(q)?;
    Ok(rows)
}

pub fn wmi_standardcimv2(
    q: &str,
) -> Result<Vec<HashMap<String, Variant>>, Box<dyn std::error::Error>> {
    let con = WMIConnection::with_namespace_path("ROOT\\standardcimv2")?;
    let rows: Vec<HashMap<String, Variant>> = con.raw_query(q)?;
    Ok(rows)
}

pub fn wmi_cimv2(q: &str) -> Result<Vec<HashMap<String, Variant>>, Box<dyn std::error::Error>> {
    let con = WMIConnection::with_namespace_path("ROOT\\CIMV2")?;
    let rows: Vec<HashMap<String, Variant>> = con.raw_query(q)?;
    Ok(rows)
}

pub fn variant_to_string(v: &Variant) -> String {
    match v {
        Variant::Empty => "Empty".to_string(),
        Variant::Null => "Null".to_string(),
        Variant::String(s) => s.clone(),
        Variant::I1(i) => i.to_string(),
        Variant::I2(i) => i.to_string(),
        Variant::I4(i) => i.to_string(),
        Variant::I8(i) => i.to_string(),
        Variant::R4(f) => f.to_string(),
        Variant::R8(f) => f.to_string(),
        Variant::Bool(b) => b.to_string(),
        Variant::UI1(u) => u.to_string(),
        Variant::UI2(u) => u.to_string(),
        Variant::UI4(u) => u.to_string(),
        Variant::UI8(u) => u.to_string(),
        Variant::Array(arr) => {
            let elems: Vec<String> = arr.iter().map(|x| variant_to_string(x)).collect();
            format!("[{}]", elems.join(", "))
        }
        Variant::Unknown(_) => "IUnknown / Temporary".to_string(),
        Variant::Object(_) => "IWbemClassWrapper".to_string(),
    }
}
