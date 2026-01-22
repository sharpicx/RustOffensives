use colored::*;

use crate::lib::advapi::*;

const IS_TEXT_UNICODE_STATISTICS: i32 = 0x0002;

pub fn list_credentials() {
    unsafe {
        let mut count: u32 = 0;
        let mut creds: *mut *mut CREDENTIALW = core::ptr::null_mut();

        let ok = CredEnumerateW(core::ptr::null(), 0, &mut count, &mut creds);
        if ok == 0 {
            return;
        }

        let slice = core::slice::from_raw_parts(creds, count as usize);

        for cred_ptr in slice {
            let cred = **cred_ptr;

            let target = u16_ptr_to_string(cred.target_name);
            let comment = u16_ptr_to_string(cred.comment);
            let username = u16_ptr_to_string(cred.user_name);
            let password = if !cred.credential_blob.is_null() && cred.credential_blob_size > 0 {
                let raw = core::slice::from_raw_parts(
                    cred.credential_blob,
                    cred.credential_blob_size as usize,
                );

                let mut flags: i32 = IS_TEXT_UNICODE_STATISTICS;
                let is_unicode = IsTextUnicode(
                    raw.as_ptr() as *const core::ffi::c_void,
                    raw.len() as i32,
                    &mut flags,
                ) != 0;

                if is_unicode {
                    let u16_slice =
                        core::slice::from_raw_parts(raw.as_ptr() as *const u16, raw.len() / 2);
                    match String::from_utf16(u16_slice) {
                        Ok(s) => s,
                        Err(_) => String::new(),
                    }
                } else {
                    raw.iter()
                        .map(|b| format!("{:02X}", b))
                        .collect::<Vec<_>>()
                        .join(" ")
                }
            } else {
                String::new()
            };

            println!("[{}] Target: {}", "+".bright_red(), target);
            if !comment.is_empty() {
                println!("    Comment: {}", comment);
            }
            println!("    Username: {}", username);
            println!("    Password: {}", password);
            println!();
        }

        CredFree(creds as *mut _);
    }
}

unsafe fn u16_ptr_to_string(ptr: *mut u16) -> String {
    if ptr.is_null() {
        return String::new();
    }

    let mut len = 0;
    loop {
        if unsafe { *ptr.add(len) } == 0 {
            break;
        }
        len += 1;
    }

    let slice = unsafe { core::slice::from_raw_parts(ptr, len) };
    String::from_utf16_lossy(slice)
}
