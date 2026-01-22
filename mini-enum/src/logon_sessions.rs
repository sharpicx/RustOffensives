use crate::lib::wmiutil::{variant_to_string, wmi_cimv2};
use std::collections::HashMap;
use wmi::Variant;

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

pub fn enum_logon_sessions() -> Result<(), Box<dyn std::error::Error>> {
    let query = "SELECT * FROM Win32_OperatingSystem";

    let results = wmi_cimv2(query)?;
    print_table(&results);

    // for (i, row) in results.iter().enumerate() {
    //     println!("Row {}:", i + 1);
    //     for (key, value) in row {
    //         println!("  {}: {:?}", key, variant_to_string(value));
    //     }
    // }

    Ok(())
}
