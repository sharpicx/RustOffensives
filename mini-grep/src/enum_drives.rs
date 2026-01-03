use colored::*;
use std::ffi::CString;
use std::os::raw::c_uint;

#[link(name = "kernel32")]
unsafe extern "system" {
    pub fn GetDriveTypeA(lpRootPathName: *const i8) -> c_uint;
}
fn drive_type_name(t: u32) -> &'static str {
    match t {
        0 => "UNKNOWN",
        1 => "NO_ROOT_DIR",
        2 => "REMOVABLE",
        3 => "FIXED",
        4 => "REMOTE",
        5 => "CDROM",
        6 => "RAMDISK",
        _ => "INVALID",
    }
}

pub fn enum_drives() {
    for letter in b'A'..=b'Z' {
        let root = format!("{}:\\", letter as char);
        let c = CString::new(root.clone()).unwrap();

        let t = unsafe { GetDriveTypeA(c.as_ptr()) };
        if t != 1 {
            let name = drive_type_name(t);
            println!(
                "[{}] Drive: {:<3} | Type: {:<10} | Code: {}",
                "+".green(),
                root,
                name,
                t
            );
        }
    }
    println!();
}
