use std::collections::HashMap;
use wmi::{Variant, WMIConnection};

fn wmi_query(q: &str) -> Result<Vec<HashMap<String, Variant>>, Box<dyn std::error::Error>> {
    let con = WMIConnection::with_namespace_path("ROOT\\standardcimv2")?;
    let rows: Vec<HashMap<String, Variant>> = con.raw_query(q)?;
    Ok(rows)
}

fn main() {
    let query = "SELECT * FROM MSFT_DNSClientCache";

    let rows = match wmi_query(query) {
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

    for row in rows {
        let entry = match row.get("Entry") {
            Some(Variant::String(s)) => s,
            _ => "",
        };
        let name = match row.get("Name") {
            Some(Variant::String(s)) => s,
            _ => "",
        };
        let data = match row.get("Data") {
            Some(Variant::String(s)) => s,
            _ => "",
        };

        println!("Entry: {}\nName: {}\nData: {}\n", entry, name, data);
    }
}
