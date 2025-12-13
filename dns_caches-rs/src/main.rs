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
        let entry = row
            .get("Entry")
            .map(|v| format!("{:?}", v))
            .unwrap_or_default();
        let name = row
            .get("Name")
            .map(|v| format!("{:?}", v))
            .unwrap_or_default();
        let data = row
            .get("Data")
            .map(|v| format!("{:?}", v))
            .unwrap_or_default();

        println!(
            "Entry: {}\n Name: {}\n Data: {}\n\n",
            entry.to_string(),
            name.to_string(),
            data.to_string()
        );
    }
}
