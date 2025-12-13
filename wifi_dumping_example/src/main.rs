use windows::{Win32::Foundation::*, Win32::NetworkManagement::WiFi::*, core::*};

fn extract_key(xml: &str) -> Option<String> {
    let start = xml.find("<keyMaterial>")?;
    let end = xml.find("</keyMaterial>")?;
    Some(xml[start + "<keyMaterial>".len()..end].trim().to_string())
}

fn get_wifi_password(profile: &str) -> Option<String> {
    unsafe {
        let mut client_handle: HANDLE = HANDLE::default();
        let mut negotiated_version: u32 = 0;

        let res = WlanOpenHandle(2, None, &mut negotiated_version, &mut client_handle);
        if res != 0 {
            return None;
        }

        let mut iface_list_ptr: *mut WLAN_INTERFACE_INFO_LIST = std::ptr::null_mut();
        let res = WlanEnumInterfaces(client_handle, None, &mut iface_list_ptr);
        if res != 0 {
            WlanCloseHandle(client_handle, None);
            return None;
        }
        let iface_list = &*iface_list_ptr;

        for i in 0..iface_list.dwNumberOfItems {
            let iface = iface_list.InterfaceInfo[i as usize];

            let mut xml_out: PWSTR = PWSTR::null();
            let mut flags: u32 = 0;

            let res = WlanGetProfile(
                client_handle,
                &iface.InterfaceGuid,
                &HSTRING::from(profile),
                None,
                &mut xml_out,
                Some(&mut flags),
                None,
            );

            if res == 0 {
                let xml = xml_out.to_string().ok()?;
                WlanFreeMemory(xml_out.0 as _);
                WlanCloseHandle(client_handle, None);
                return extract_key(&xml);
            }
        }

        WlanCloseHandle(client_handle, None);
        None
    }
}

fn main() {
    if let Some(pass) = get_wifi_password("gggggggg") {
        println!("{}", pass);
    } else {
        println!("No key found or error");
    }
}
