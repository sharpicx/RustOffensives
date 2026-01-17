use core::slice::from_raw_parts;
use ntapi::ntldr::LDR_DATA_TABLE_ENTRY;
use winapi::um::winnt::{IMAGE_DOS_HEADER, IMAGE_EXPORT_DIRECTORY, IMAGE_NT_HEADERS64};

const IMAGE_NT_SIGNATURE: u32 = 0x0000_4550;
const IMAGE_DOS_SIGNATURE: u16 = 0x5A4D;
const IMAGE_DIRECTORY_ENTRY_EXPORT: usize = 0;

fn dbj2_hash(buffer: &[u8]) -> Result<u32, &str> {
    let mut hsh: u32 = 5381;
    let mut iter: usize = 0;
    let mut cur: u8;
    while iter < buffer.len() {
        cur = buffer[iter];
        if cur == 0 {
            iter += 1;
            continue;
        }
        if cur >= ('a' as u8) {
            cur -= 0x20;
        }
        hsh = ((hsh << 5).wrapping_add(hsh)) + cur as u32;
        iter += 1;
    }
    Ok(hsh)
}

pub fn get_module_base_by_hash(target_hash: u32) -> *mut u8 {
    unsafe {
        let peb: *const ntapi::ntpebteb::PEB;
        core::arch::asm!("mov {}, gs:[0x60]", out(reg) peb);
        let ldr = (*peb).Ldr;
        let head = &(*ldr).InLoadOrderModuleList as *const _ as *mut _;
        let mut entry = (*ldr).InLoadOrderModuleList.Flink;
        while entry != head {
            let table = entry as *const LDR_DATA_TABLE_ENTRY;
            let name = (*table).BaseDllName;
            if !name.Buffer.is_null() && name.Length > 0 {
                let byte_len = name.Length as usize;
                let bytes = core::slice::from_raw_parts(name.Buffer as *const u8, byte_len);

                if let Ok(h) = dbj2_hash(bytes) {
                    if h == target_hash {
                        return (*table).DllBase as *mut u8;
                    }
                }
            }
            entry = (*entry).Flink;
        }
        core::ptr::null_mut()
    }
}

pub fn get_export_by_hash(module_base: *const u8, export_name_hash: u32) -> Option<usize> {
    unsafe {
        if module_base.is_null() {
            return None;
        }
        let dos = module_base as *const IMAGE_DOS_HEADER;
        if (*dos).e_magic != IMAGE_DOS_SIGNATURE {
            return None;
        }
        let nt = module_base.add((*dos).e_lfanew as usize) as *const IMAGE_NT_HEADERS64;
        if (*nt).Signature != IMAGE_NT_SIGNATURE {
            return None;
        }
        let export_dir_rva = (*nt).OptionalHeader.DataDirectory
            [IMAGE_DIRECTORY_ENTRY_EXPORT as usize]
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
            if name_ptr.is_null() {
                continue;
            }
            let name_bytes = core::ffi::CStr::from_ptr(name_ptr).to_bytes();
            if let Ok(hash) = dbj2_hash(name_bytes) {
                if hash == export_name_hash {
                    let ordinal = ordinals[i] as usize;
                    let func_rva = functions.get(ordinal)?;
                    return Some(module_base as usize + *func_rva as usize);
                }
            }
        }
        None
    }
}
