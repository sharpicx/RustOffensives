use core::slice::from_raw_parts;
use ntapi::ntldr::LDR_DATA_TABLE_ENTRY;
use std::ffi::CStr;
use winapi::um::winnt::*;

#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
pub struct LIST_ENTRY {
    pub Flink: *mut LIST_ENTRY,
}

pub const IMAGE_DOS_SIGNATURE: u16 = 0x5A4D;
pub const IMAGE_DIRECTORY_ENTRY_EXPORT: usize = 0;

pub fn current_peb() -> *const ntapi::ntpebteb::PEB {
    unsafe { ntapi::ntpsapi::NtCurrentPeb() }
}

pub fn hash(buffer: &[u8]) -> Result<u32, &str> {
    let mut hsh: u32 = 0x1337BEEF;
    let secret_key: u32 = 0x7A;
    let mut iter: usize = 0;
    while iter < buffer.len() {
        let mut cur = buffer[iter];
        if cur == 0 {
            iter += 1;
            continue;
        }
        if cur >= b'a' && cur <= b'z' {
            cur -= 0x20;
        }
        hsh = (hsh.wrapping_shl(5).wrapping_add(hsh)) ^ (cur as u32 ^ secret_key);
        iter += 1;
    }
    Ok(hsh)
}

pub fn get_module_base(target_hash: u32) -> *mut u8 {
    unsafe {
        let peb = current_peb();
        if peb.is_null() {
            return core::ptr::null_mut();
        }
        let ldr = (*peb).Ldr;
        if ldr.is_null() {
            return core::ptr::null_mut();
        }
        let head = &(*ldr).InLoadOrderModuleList as *const _ as *const LIST_ENTRY;
        let mut cur = (*head).Flink;
        while !std::ptr::eq(cur, head as *mut LIST_ENTRY) {
            let entry = cur as *const LDR_DATA_TABLE_ENTRY;
            let name = (*entry).BaseDllName;
            if !name.Buffer.is_null() && name.Length != 0 {
                let bytes = from_raw_parts(name.Buffer as *const u8, name.Length as usize);
                if let Ok(h) = hash(bytes) {
                    if h == target_hash {
                        return (*entry).DllBase as *mut u8;
                    }
                }
            }
            cur = (*cur).Flink;
        }
        core::ptr::null_mut()
    }
}

pub fn get_export(module_base: *const u8, export_name_hash: u32) -> Option<usize> {
    unsafe {
        if module_base.is_null() {
            return None;
        }
        let dos = module_base as *const IMAGE_DOS_HEADER;
        if (*dos).e_magic != IMAGE_DOS_SIGNATURE {
            return None;
        }
        let nt = module_base.add((*dos).e_lfanew as usize) as *const IMAGE_NT_HEADERS64;
        let export_dir_rva = (*nt).OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_EXPORT]
            .VirtualAddress as usize;
        if export_dir_rva == 0 {
            return None;
        }
        let export_dir = module_base.add(export_dir_rva) as *const IMAGE_EXPORT_DIRECTORY;
        let names = from_raw_parts(
            module_base.add((*export_dir).AddressOfNames as usize) as *const u32,
            (*export_dir).NumberOfNames as usize,
        );
        let ordinals = from_raw_parts(
            module_base.add((*export_dir).AddressOfNameOrdinals as usize) as *const u16,
            (*export_dir).NumberOfNames as usize,
        );
        let functions = from_raw_parts(
            module_base.add((*export_dir).AddressOfFunctions as usize) as *const u32,
            (*export_dir).NumberOfFunctions as usize,
        );
        for i in 0..names.len() {
            let name_ptr = module_base.add(names[i] as usize) as *const i8;
            let name_bytes = CStr::from_ptr(name_ptr).to_bytes();
            if let Ok(hash) = hash(name_bytes) {
                if hash == export_name_hash {
                    let ordinal = ordinals[i] as usize;
                    return Some(module_base as usize + functions[ordinal] as usize);
                }
            }
        }
        None
    }
}
