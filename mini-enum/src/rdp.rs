use crate::helper::log_success;
use std::ffi::c_void;
use windows_sys::Win32::Foundation::ERROR_NO_MORE_ITEMS;
use windows_sys::Win32::System::Registry::{
    HKEY, HKEY_USERS, KEY_READ, RRF_RT_REG_SZ, RegCloseKey, RegEnumKeyExW, RegGetValueW,
    RegOpenKeyExW,
};

pub struct RdpConnection {
    pub remote_host: String,
    pub username_hint: String,
}

pub struct RdpSavedConnection {
    pub sid: String,
    pub connections: Vec<RdpConnection>,
}

fn get_user_sids() -> Vec<String> {
    unsafe {
        pub type HKEY = *mut c_void;
        let mut hkey: HKEY = core::ptr::null_mut();
        let empty: [u16; 1] = [0];

        if RegOpenKeyExW(HKEY_USERS, empty.as_ptr(), 0, KEY_READ, &mut hkey) != 0 {
            return Vec::new();
        }

        let mut out = Vec::new();
        let mut index = 0;

        loop {
            let mut buf = [0u16; 260];
            let mut len = buf.len() as u32;

            let r = RegEnumKeyExW(
                hkey,
                index,
                buf.as_mut_ptr(),
                &mut len,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );

            if r == ERROR_NO_MORE_ITEMS {
                break;
            }
            if r != 0 {
                break;
            }

            out.push(String::from_utf16_lossy(&buf[..len as usize]));
            index += 1;
        }

        RegCloseKey(hkey);
        out
    }
}

fn get_subkeys(base: &str) -> Option<Vec<String>> {
    unsafe {
        let mut hkey: HKEY = core::ptr::null_mut();
        let wpath: Vec<u16> = base.encode_utf16().chain([0]).collect();

        if RegOpenKeyExW(HKEY_USERS, wpath.as_ptr(), 0, KEY_READ, &mut hkey) != 0 {
            return None;
        }

        let mut out = Vec::new();
        let mut index = 0;

        loop {
            let mut buf = [0u16; 260];
            let mut len = buf.len() as u32;

            let r = RegEnumKeyExW(
                hkey,
                index,
                buf.as_mut_ptr(),
                &mut len,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );

            if r == ERROR_NO_MORE_ITEMS {
                break;
            }
            if r != 0 {
                break;
            }

            out.push(String::from_utf16_lossy(&buf[..len as usize]));
            index += 1;
        }

        RegCloseKey(hkey);
        Some(out)
    }
}

fn get_string_value(path: &str, name: &str) -> Option<String> {
    unsafe {
        let full = format!("{}\0", path).encode_utf16().collect::<Vec<u16>>();
        let wname = format!("{}\0", name).encode_utf16().collect::<Vec<u16>>();

        let mut buf = [0u16; 260];
        let mut bytes = (buf.len() * 2) as u32;

        let r = RegGetValueW(
            HKEY_USERS,
            full.as_ptr(),
            wname.as_ptr(),
            RRF_RT_REG_SZ,
            core::ptr::null_mut(),
            buf.as_mut_ptr() as *mut _,
            &mut bytes,
        );

        if r != 0 {
            return None;
        }

        let len = (bytes as usize / 2).saturating_sub(1);
        Some(String::from_utf16_lossy(&buf[..len]))
    }
}

fn get_rdp_saved_creds() -> Vec<RdpSavedConnection> {
    let mut out = Vec::new();
    let sids = get_user_sids();

    for sid in sids {
        if !sid.starts_with("S-1-5") || sid.ends_with("_Classes") {
            continue;
        }

        let base = format!(
            "{}\\Software\\Microsoft\\Terminal Server Client\\Servers",
            sid
        );

        let hosts = match get_subkeys(&base) {
            Some(v) => v,
            None => continue,
        };

        if hosts.is_empty() {
            continue;
        }

        let mut conns = Vec::new();
        for host in hosts {
            let host_key = format!("{}\\{}", base, host);

            let hint = get_string_value(&host_key, "UsernameHint").unwrap_or_default();

            conns.push(RdpConnection {
                remote_host: host,
                username_hint: hint,
            });
        }

        out.push(RdpSavedConnection {
            sid,
            connections: conns,
        });
    }

    out
}

pub fn now() {
    let data = get_rdp_saved_creds();
    for entry in data {
        if entry.connections.is_empty() {
            continue;
        }
        log_success(&format!(
            "Saved RDP Connection Information ({})\n",
            entry.sid
        ));
        println!("    RemoteHost                         UsernameHint");
        println!("    ----------                         ------------");
        for c in entry.connections {
            let mut host = c.remote_host.clone();
            while host.len() < 35 {
                host.push(' ');
            }
            println!("    {}{}", host, c.username_hint);
        }
        println!();
    }
}
