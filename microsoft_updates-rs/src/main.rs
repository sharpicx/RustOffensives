use chrono::{Duration, NaiveDate};
use regex::Regex;
use std::collections::HashMap;
use std::mem::ManuallyDrop;
use std::ptr::null_mut;
use widestring::U16CStr;
use windows::Win32::System::Com::*;
use windows::Win32::System::Variant::*;
use windows::core::*;

type CLSID = GUID;

fn main() -> Result<()> {
    unsafe {
        let mut updates: HashMap<String, Vec<(String, String, String)>> = HashMap::new();
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
        let clsid_microsoft_update_searcher: CLSID =
            CLSIDFromProgID(&HSTRING::from("Microsoft.Update.Searcher"))?;
        let searcher: IDispatch = CoCreateInstance(
            &clsid_microsoft_update_searcher,
            None,
            CLSCTX_INPROC_SERVER | CLSCTX_LOCAL_SERVER,
        )?;
        let mut dispid = 0;
        let get_total_history_count_utf16: Vec<u16> = "GetTotalHistoryCount"
            .encode_utf16()
            .chain(Some(0))
            .collect();
        let get_total_history_count_pcwstr: PCWSTR = PCWSTR(get_total_history_count_utf16.as_ptr());
        searcher.GetIDsOfNames(
            &GUID::default(),
            &get_total_history_count_pcwstr,
            1,
            0,
            &mut dispid,
        )?;
        let mut result = VARIANT::default();
        let dispparams = DISPPARAMS {
            rgvarg: null_mut(),
            rgdispidNamedArgs: null_mut(),
            cArgs: 0,
            cNamedArgs: 0,
        };
        searcher.Invoke(
            dispid,
            &GUID::default(),
            0,
            DISPATCH_METHOD,
            &dispparams,
            Some(&mut result),
            None,
            None,
        )?;
        // https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/System/Variant/union.VARIANT_0_0_0.html
        // $ ../../Seatbelt.exe MicrosoftUpdates | grep '2025' | wc -l
        // 982
        let count = result.Anonymous.Anonymous.Anonymous.lVal; // 982
        VariantClear(&mut result).ok();
        // println!("{}", count);
        let query_history_utf16: Vec<u16> = "QueryHistory".encode_utf16().chain(Some(0)).collect();
        let pcwstr_query = PCWSTR(query_history_utf16.as_ptr());
        let mut dispid_query = 0;
        searcher.GetIDsOfNames(&GUID::default(), &pcwstr_query, 1, 0, &mut dispid_query)?;
        let mut args = [VARIANT::from(count), VARIANT::from(0)];
        let mut results_var = VARIANT::default();
        let dispparams_query = DISPPARAMS {
            rgvarg: args.as_mut_ptr(),
            rgdispidNamedArgs: null_mut(),
            cArgs: args.len() as u32,
            cNamedArgs: 0,
        };
        searcher.Invoke(
            dispid_query,
            &GUID::default(),
            0,
            DISPATCH_METHOD,
            &dispparams_query,
            Some(&mut results_var),
            None,
            None,
        )?;

        let results_dispatch = &results_var.Anonymous.Anonymous.Anonymous.pdispVal;
        if let Some(dispatch) = results_dispatch.as_ref() {
            let item_name_utf16: Vec<u16> = "Item".encode_utf16().chain(Some(0)).collect();
            let mut dispid_item = 0;
            dispatch.GetIDsOfNames(
                &GUID::default(),
                &PCWSTR(item_name_utf16.as_ptr()),
                1,
                0,
                &mut dispid_item,
            )?;
            let reg = Regex::new(r"KB\d+").unwrap();
            for i in 0..count {
                let mut item_var = VARIANT::default();
                let mut item_args = [VARIANT::from(i)];
                let item_dispparams = DISPPARAMS {
                    rgvarg: item_args.as_mut_ptr(),
                    rgdispidNamedArgs: std::ptr::null_mut(),
                    cArgs: 1,
                    cNamedArgs: 0,
                };

                dispatch.Invoke(
                    dispid_item,
                    &GUID::default(),
                    0,
                    DISPATCH_PROPERTYGET,
                    &item_dispparams,
                    Some(&mut item_var),
                    None,
                    None,
                )?;
                let item_dispatch = &item_var.Anonymous.Anonymous.Anonymous.pdispVal;
                if let Some(prop_dispatch) = item_dispatch.as_ref() {
                    let properties = ["Title", "Date", "Description", "ClientApplicationID"];
                    let mut prop_values = Vec::new();

                    for &prop_name in &properties {
                        let prop_utf16: Vec<u16> =
                            prop_name.encode_utf16().chain(Some(0)).collect();
                        let mut dispid_prop = 0;
                        prop_dispatch.GetIDsOfNames(
                            &GUID::default(),
                            &PCWSTR(prop_utf16.as_ptr()),
                            1,
                            0,
                            &mut dispid_prop,
                        )?;

                        let mut prop_var = VARIANT::default();
                        let dispparams_prop = DISPPARAMS {
                            rgvarg: std::ptr::null_mut(),
                            rgdispidNamedArgs: std::ptr::null_mut(),
                            cArgs: 0,
                            cNamedArgs: 0,
                        };

                        prop_dispatch.Invoke(
                            dispid_prop,
                            &GUID::default(),
                            0,
                            DISPATCH_PROPERTYGET,
                            &dispparams_prop,
                            Some(&mut prop_var),
                            None,
                            None,
                        )?;

                        let value = match prop_var.Anonymous.Anonymous.vt {
                            VT_BSTR => {
                                let bstr: &ManuallyDrop<BSTR> =
                                    &prop_var.Anonymous.Anonymous.Anonymous.bstrVal;
                                if !bstr.is_empty() {
                                    U16CStr::from_ptr_str(bstr.as_ptr()).to_string_lossy()
                                } else {
                                    String::new()
                                }
                            }
                            VT_DATE => {
                                let date_f64 = prop_var.Anonymous.Anonymous.Anonymous.date;
                                let base_date = NaiveDate::from_ymd_opt(1899, 12, 30)
                                    .expect("invalid base date")
                                    .and_hms_opt(0, 0, 0)
                                    .expect("invalid time");
                                let duration =
                                    Duration::milliseconds((date_f64 * 86_400_000.0) as i64);
                                let datetime = base_date + duration;
                                let formatted =
                                    datetime.format("%-d/%-m/%Y %-I:%M:%S %p").to_string(); // indonesia
                                formatted.to_string()
                            }
                            _ => String::new(),
                        };
                        prop_values.push(value);
                        VariantClear(&mut prop_var).ok();
                    }
                    let title = &prop_values[0];
                    let date = &prop_values[1];
                    let client = &prop_values[3];
                    let hotfix_id = reg
                        .find(title)
                        .map_or(String::new(), |m| m.as_str().to_string());
                    updates.entry(hotfix_id).or_default().push((
                        date.clone(),
                        client.clone(),
                        title.clone(),
                    ));
                }
                VariantClear(&mut item_var).ok();
            }
            for (kb, entries) in updates.iter() {
                for (i, (date, client, title)) in entries.iter().enumerate() {
                    if i == 0 {
                        println!("  {:<10} {:<23} {:<23} {}", kb, date, client, title);
                    } else {
                        println!("  {:<10} {:<23} {:<23} {}", "", date, client, title);
                    }
                }
            }
        }
        VariantClear(&mut results_var).ok();
        CoUninitialize();
    }

    Ok(())
}
