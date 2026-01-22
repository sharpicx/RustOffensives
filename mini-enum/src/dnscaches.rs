use crate::lib::wmiutil::wmi_standardcimv2;
use wmi::Variant;

pub fn enum_dns_caches() {
    let query = "SELECT * FROM MSFT_DNSClientCache";
    let rows = match wmi_standardcimv2(query) {
        Ok(r) => r,
        Err(e) => {
            if e.to_string().contains("Invalid namespace") {
                eprintln!(
                    "  [X] 'MSFT_DNSClientCache' WMI class unavailable (minimum supported Windows 8/2012)"
                );
                return;
            } else {
                eprintln!("Error: {}", e);
                return;
            }
        }
    };
    for (i, row) in rows.iter().enumerate() {
        let entry = match row.get("Entry") {
            Some(Variant::String(s)) => s,
            _ => "N/A",
        };
        let name = match row.get("Name") {
            Some(Variant::String(s)) if !s.trim().is_empty() => s,
            _ => "N/A",
        };
        let data = match row.get("Data") {
            Some(Variant::String(s)) => s,
            _ => "N/A",
        };
        println!("ID: {}", i + 1);
        println!("├── Entry: {}", entry);
        println!("├── Name: {}", name);
        println!("└── Data: {}", data);
        println!();
    }
}
