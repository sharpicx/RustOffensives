use std::collections::HashMap;
use wmi::{Variant, WMIConnection};

fn variant_to_string(v: &Variant) -> String {
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

fn print_table(rows: &[HashMap<String, Variant>]) {
    println!();
    if rows.is_empty() {
        println!("No data");
        return;
    }
    let mut key_width = "Property".len();
    let mut value_width = "Value".len();
    for row in rows {
        for (k, v) in row {
            key_width = key_width.max(k.len());
            value_width = value_width.max(variant_to_string(v).len());
        }
    }
    let total_width = key_width + value_width + 5;
    println!("+{}+", "-".repeat(total_width));
    println!(
        "| {:<key_width$} | {:<value_width$} |",
        "Property",
        "Value",
        key_width = key_width,
        value_width = value_width
    );
    println!(
        "+{}+{}+",
        "-".repeat(key_width + 2),
        "-".repeat(value_width + 2)
    );
    for row in rows {
        for (k, v) in row {
            println!(
                "| {:<key_width$} | {:<value_width$} |",
                k,
                variant_to_string(v),
                key_width = key_width,
                value_width = value_width
            );
        }
        println!("+{}+", "-".repeat(total_width));
    }
    println!();
}

fn wmi_query(q: &str) -> Result<Vec<HashMap<String, Variant>>, Box<dyn std::error::Error>> {
    let con = WMIConnection::with_namespace_path("ROOT\\CIMV2")?;
    let rows: Vec<HashMap<String, Variant>> = con.raw_query(q)?;
    Ok(rows)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let query = "SELECT * FROM Win32_OperatingSystem";

    let results = wmi_query(query)?;
    print_table(&results);

    // for (i, row) in results.iter().enumerate() {
    //     println!("Row {}:", i + 1);
    //     for (key, value) in row {
    //         println!("  {}: {:?}", key, variant_to_string(value));
    //     }
    // }

    Ok(())
}
