use anyhow::Result;
use std::ffi::CStr;
use std::ffi::c_void;
use std::mem::{size_of, transmute};
use std::ptr::null_mut;
use winapi::shared::ntdef::{ANSI_STRING, UNICODE_STRING};
use winapi::um::winnt::*;

use crate::helper::{ProcessParametersStore, fix_args, restore_args};
use crate::walker::{get_export_by_hash, get_module_base_by_hash};

pub fn load(raw_data: Vec<u8>, target_args: Vec<String>) -> Result<()> {
    unsafe {
        let raw_ptr = raw_data.as_ptr() as *mut u8;
        let ntdll = get_module_base_by_hash(0x1edab0ed);
        let nt_alloc_addr = get_export_by_hash(ntdll, 0xf783b8ec).unwrap();
        let nt_protect_addr = get_export_by_hash(ntdll, 0x50e92888).unwrap();
        let nt_allocate_virtual_memory: extern "system" fn(
            HANDLE,
            *mut *mut c_void,
            usize,
            *mut usize,
            u32,
            u32,
        ) -> i32 = transmute(nt_alloc_addr);
        let nt_protect_virtual_memory: extern "system" fn(
            HANDLE,
            *mut *mut c_void,
            *mut usize,
            u32,
            *mut u32,
        ) -> i32 = transmute(nt_protect_addr);
        let dos = raw_ptr as *mut IMAGE_DOS_HEADER;
        let nt = raw_ptr.add((*dos).e_lfanew as usize) as *mut IMAGE_NT_HEADERS64;
        let mut image_base: *mut c_void = std::ptr::null_mut();
        let mut size = (*nt).OptionalHeader.SizeOfImage as usize;
        nt_allocate_virtual_memory(
            -1isize as HANDLE,
            &mut image_base,
            0,
            &mut size,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        );
        copy(raw_ptr, image_base, size);
        reloc(
            image_base as *mut u8,
            size as u64,
            image_base as u64,
            (*nt).OptionalHeader.ImageBase,
        );
        resolve(image_base as *mut u8)?;
        update(target_args.clone())?;
        force_update(target_args.clone())?;
        let full_command_line = target_args[1..].join(" ");
        let c_full_payload = std::ffi::CString::new(full_command_line.clone()).unwrap();
        let mut arg_store = ProcessParametersStore {
            commandline_len_orig: 0,
            commandline_max_orig: 0,
            commandline_ptr_orig: std::ptr::null_mut(),
        };
        fix_args(&mut arg_store, c_full_payload.as_ptr());
        protect(image_base, nt_protect_virtual_memory);
        let entry_ptr = (image_base as usize + (*nt).OptionalHeader.AddressOfEntryPoint as usize)
            as *const c_void;
        if ((*nt).FileHeader.Characteristics & 0x2000) != 0 {
            let dll_main: extern "system" fn(*mut c_void, u32, *mut c_void) -> i32 =
                transmute(entry_ptr);
            dll_main(image_base, 1, std::ptr::null_mut());
        } else {
            let exe_main: extern "C" fn(i32, *mut *mut i8) = transmute(entry_ptr);
            exe_main(1, null_mut());
        }
        restore_args(&mut arg_store);
        Ok(())
    }
}

fn copy(source_buffer: *mut u8, base_address: *mut c_void, view_size: usize) -> bool {
    unsafe {
        let dos = source_buffer as *mut IMAGE_DOS_HEADER;
        let nt = source_buffer.add((*dos).e_lfanew as usize) as *mut IMAGE_NT_HEADERS64;
        let headers_size = (*nt).OptionalHeader.SizeOfHeaders as usize;

        std::ptr::copy_nonoverlapping(source_buffer, base_address as *mut u8, headers_size);

        let mut sec = get(nt);
        for _ in 0..(*nt).FileHeader.NumberOfSections {
            if (*sec).SizeOfRawData > 0 {
                let dst = (base_address as *mut u8).add((*sec).VirtualAddress as usize);
                let src = source_buffer.add((*sec).PointerToRawData as usize);
                let mut size = (*sec).SizeOfRawData as usize;

                if ((*sec).VirtualAddress as usize + size) > view_size {
                    size = view_size - (*sec).VirtualAddress as usize;
                }
                std::ptr::copy_nonoverlapping(src, dst, size);
            }
            sec = sec.add(1);
        }
    }
    true
}

fn reloc(base: *mut u8, _size: u64, new_base: u64, old_base: u64) -> bool {
    unsafe {
        let delta = new_base.wrapping_sub(old_base);
        if delta == 0 {
            return true;
        }

        let dos = base as *mut IMAGE_DOS_HEADER;
        let nt = base.add((*dos).e_lfanew as usize) as *mut IMAGE_NT_HEADERS64;
        let reloc_dir = (*nt).OptionalHeader.DataDirectory[5];

        if reloc_dir.VirtualAddress == 0 {
            return true;
        }

        let mut processed = 0;
        let mut block = base.add(reloc_dir.VirtualAddress as usize) as *mut IMAGE_BASE_RELOCATION;

        while processed < reloc_dir.Size as usize && (*block).SizeOfBlock > 0 {
            let count = ((*block).SizeOfBlock as usize - size_of::<IMAGE_BASE_RELOCATION>()) / 2;
            let entries = (block as usize + size_of::<IMAGE_BASE_RELOCATION>()) as *const u16;

            for i in 0..count {
                let entry_val = *entries.add(i);
                let type_ = entry_val >> 12;
                let offset = entry_val & 0xFFF;

                if type_ == 10 {
                    let patch =
                        base.add((*block).VirtualAddress as usize + offset as usize) as *mut usize;
                    *patch = (*patch).wrapping_add(delta as usize);
                }
            }
            processed += (*block).SizeOfBlock as usize;
            block = (block as usize + (*block).SizeOfBlock as usize) as PIMAGE_BASE_RELOCATION;
        }
    }
    true
}

fn protect(
    base_address: *mut c_void,
    nt_protect: extern "system" fn(HANDLE, *mut *mut c_void, *mut usize, u32, *mut u32) -> i32,
) -> bool {
    unsafe {
        let dos = base_address as *mut IMAGE_DOS_HEADER;
        let nt = (base_address as *mut u8).add((*dos).e_lfanew as usize) as *mut IMAGE_NT_HEADERS64;
        let mut sec = get(nt);

        for _ in 0..(*nt).FileHeader.NumberOfSections {
            let char = (*sec).Characteristics;
            let mut prot = PAGE_NOACCESS;

            if (char & IMAGE_SCN_MEM_EXECUTE) != 0 {
                prot = if (char & IMAGE_SCN_MEM_WRITE) != 0 {
                    PAGE_EXECUTE_READWRITE
                } else {
                    PAGE_EXECUTE_READ
                };
            } else if (char & IMAGE_SCN_MEM_WRITE) != 0 {
                prot = PAGE_READWRITE;
            } else if (char & IMAGE_SCN_MEM_READ) != 0 {
                prot = PAGE_READONLY;
            }

            let mut addr =
                (base_address as *mut u8).add((*sec).VirtualAddress as usize) as *mut c_void;
            let mut size = *(*sec).Misc.VirtualSize() as usize;
            if size == 0 {
                size = (*sec).SizeOfRawData as usize;
            }

            let mut old = 0u32;
            nt_protect(-1isize as HANDLE, &mut addr, &mut size, prot, &mut old);
            sec = sec.add(1);
        }
    }
    true
}

fn get(nt_header: *mut IMAGE_NT_HEADERS64) -> *mut IMAGE_SECTION_HEADER {
    unsafe {
        let ptr = nt_header as *mut u8;
        let offset = (&(*nt_header).OptionalHeader as *const _ as usize) - (nt_header as usize);
        let size_of_optional = (*nt_header).FileHeader.SizeOfOptionalHeader as usize;
        ptr.add(offset + size_of_optional) as *mut IMAGE_SECTION_HEADER
    }
}

fn resolve(base_address: *mut u8) -> Result<()> {
    unsafe {
        let ntdll = get_module_base_by_hash(0x1edab0ed);
        let ldr_load_dll_addr = get_export_by_hash(ntdll, 0x9e456a43);
        let ldr_get_proc_addr = get_export_by_hash(ntdll, 0xfce76bb6);
        let ldr_load_dll: extern "system" fn(
            *mut u16,
            *mut u32,
            *mut UNICODE_STRING,
            *mut *mut c_void,
        ) -> i32 = transmute(ldr_load_dll_addr.unwrap());
        let ldr_get_procedure_address: extern "system" fn(
            *mut c_void,
            *mut ANSI_STRING,
            u32,
            *mut *mut c_void,
        ) -> i32 = transmute(ldr_get_proc_addr.unwrap());
        let dos = base_address as *mut IMAGE_DOS_HEADER;
        let nt = base_address.add((*dos).e_lfanew as usize) as *mut IMAGE_NT_HEADERS64;
        let import_dir = (*nt).OptionalHeader.DataDirectory[1];
        if import_dir.VirtualAddress == 0 {
            return Ok(());
        }
        let mut import_desc =
            base_address.add(import_dir.VirtualAddress as usize) as *mut IMAGE_IMPORT_DESCRIPTOR;
        while (*import_desc).Name != 0 {
            let dll_name_ptr = base_address.add((*import_desc).Name as usize) as *const i8;
            let dll_name_str = CStr::from_ptr(dll_name_ptr).to_str()?;
            let mut dll_name_utf16: Vec<u16> = dll_name_str
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();
            let mut dll_unicode = UNICODE_STRING {
                Length: ((dll_name_utf16.len() - 1) * 2) as u16,
                MaximumLength: (dll_name_utf16.len() * 2) as u16,
                Buffer: dll_name_utf16.as_mut_ptr(),
            };
            let mut h_module: *mut c_void = std::ptr::null_mut();
            ldr_load_dll(
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut dll_unicode,
                &mut h_module,
            );
            let mut thunk = base_address.add((*import_desc).FirstThunk as usize) as *mut usize;
            let oft = *(*import_desc).u.OriginalFirstThunk();
            let mut orig_thunk = if oft != 0 {
                base_address.add(oft as usize) as *mut usize
            } else {
                thunk
            };
            while *orig_thunk != 0 {
                let mut func_addr: *mut c_void = std::ptr::null_mut();
                if (*orig_thunk & (1 << 63)) != 0 {
                    ldr_get_procedure_address(
                        h_module,
                        std::ptr::null_mut(),
                        (*orig_thunk & 0xFFFF) as u32,
                        &mut func_addr,
                    );
                } else {
                    let ibn = base_address.add(*orig_thunk) as *mut IMAGE_IMPORT_BY_NAME;
                    let mut ansi_str = ANSI_STRING {
                        Length: CStr::from_ptr((*ibn).Name.as_ptr()).to_bytes().len() as u16,
                        MaximumLength: CStr::from_ptr((*ibn).Name.as_ptr()).to_bytes().len() as u16,
                        Buffer: (*ibn).Name.as_ptr() as *mut i8,
                    };
                    ldr_get_procedure_address(h_module, &mut ansi_str, 0, &mut func_addr);
                }
                *thunk = func_addr as usize;
                thunk = thunk.add(1);
                orig_thunk = orig_thunk.add(1);
            }
            import_desc = import_desc.add(1);
        }
        Ok(())
    }
}

fn update(args: Vec<String>) -> Result<()> {
    unsafe {
        let ntdll = get_module_base_by_hash(0x1edab0ed);
        let nt_query_addr = get_export_by_hash(ntdll, 0x8cdc5dc2).unwrap();
        let nt_query_info: extern "system" fn(HANDLE, u32, *mut c_void, u32, *mut u32) -> i32 =
            transmute(nt_query_addr);
        let mut pbi = std::mem::zeroed::<ntapi::ntpsapi::PROCESS_BASIC_INFORMATION>();
        nt_query_info(
            -1isize as HANDLE,
            0,
            &mut pbi as *mut _ as *mut c_void,
            size_of::<ntapi::ntpsapi::PROCESS_BASIC_INFORMATION>() as u32,
            std::ptr::null_mut(),
        );
        let peb = pbi.PebBaseAddress;
        let proc_params = (*peb).ProcessParameters;
        let cmd_line_plain = args.join(" ");
        let utf16_cmd: Vec<u16> = cmd_line_plain
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let mut new_buffer: *mut c_void = std::ptr::null_mut();
        let mut size = utf16_cmd.len() * 2;

        let nt_alloc: extern "system" fn(
            HANDLE,
            *mut *mut c_void,
            usize,
            *mut usize,
            u32,
            u32,
        ) -> i32 = transmute(get_export_by_hash(ntdll, 0xf783b8ec).unwrap());
        nt_alloc(
            -1isize as HANDLE,
            &mut new_buffer,
            0,
            &mut size,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        );
        std::ptr::copy_nonoverlapping(utf16_cmd.as_ptr(), new_buffer as *mut u16, utf16_cmd.len());
        (*proc_params).CommandLine.Buffer = new_buffer as *mut u16;
        (*proc_params).CommandLine.Length = ((utf16_cmd.len() - 1) * 2) as u16;
        (*proc_params).CommandLine.MaximumLength = (utf16_cmd.len() * 2) as u16;
        let image_path = &args[0];
        let utf16_path: Vec<u16> = image_path
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let mut path_buffer: *mut c_void = std::ptr::null_mut();
        let mut path_size = utf16_path.len() * 2;
        nt_alloc(
            -1isize as HANDLE,
            &mut path_buffer,
            0,
            &mut path_size,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        );
        std::ptr::copy_nonoverlapping(
            utf16_path.as_ptr(),
            path_buffer as *mut u16,
            utf16_path.len(),
        );
        (*proc_params).ImagePathName.Buffer = path_buffer as *mut u16;
        (*proc_params).ImagePathName.Length = ((utf16_path.len() - 1) * 2) as u16;
        (*proc_params).ImagePathName.MaximumLength = (utf16_path.len() * 2) as u16;
        Ok(())
    }
}

fn force_update(args: Vec<String>) -> Result<(*mut u16, *mut i8)> {
    unsafe {
        let cmd_line_plain = args.join(" ") + " ";
        let utf16_cmd: Vec<u16> = cmd_line_plain
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let ansi_cmd = format!("{}\0", cmd_line_plain);
        let ntdll = get_module_base_by_hash(0x1edab0ed);
        let nt_alloc: extern "system" fn(
            HANDLE,
            *mut *mut c_void,
            usize,
            *mut usize,
            u32,
            u32,
        ) -> i32 = transmute(get_export_by_hash(ntdll, 0xf783b8ec).unwrap());
        let mut new_u16: *mut c_void = std::ptr::null_mut();
        let mut size_u16 = utf16_cmd.len() * 2;
        nt_alloc(
            -1isize as HANDLE,
            &mut new_u16,
            0,
            &mut size_u16,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        );
        std::ptr::copy_nonoverlapping(utf16_cmd.as_ptr(), new_u16 as *mut u16, utf16_cmd.len());
        let mut new_a: *mut c_void = std::ptr::null_mut();
        let mut size_a = ansi_cmd.len();
        nt_alloc(
            -1isize as HANDLE,
            &mut new_a,
            0,
            &mut size_a,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        );
        std::ptr::copy_nonoverlapping(ansi_cmd.as_ptr(), new_a as *mut u8, ansi_cmd.len());
        let kernelbase = get_module_base_by_hash(0x3ebb38b);
        let nt_protect_addr = get_export_by_hash(ntdll, 0x50e92888).unwrap();
        let nt_protect: extern "system" fn(
            HANDLE,
            *mut *mut c_void,
            *mut usize,
            u32,
            *mut u32,
        ) -> i32 = transmute(nt_protect_addr);
        if let Some(func_w) = get_export_by_hash(kernelbase, 0x32794b03) {
            apply(func_w as *mut u8, new_u16, nt_protect);
        }
        if let Some(func_a) = get_export_by_hash(kernelbase, 0x32794aed) {
            apply(func_a as *mut u8, new_a, nt_protect);
        }
        let kernelbase = get_module_base_by_hash(0x3ebb38b);
        if !kernelbase.is_null() {
            if let Some(func_w) = get_export_by_hash(kernelbase, 0x32794b03) {
                apply(func_w as *mut u8, new_u16, nt_protect);
            }
            if let Some(func_a) = get_export_by_hash(kernelbase, 0x32794aed) {
                apply(func_a as *mut u8, new_a, nt_protect);
            }
        }
        let crt_base = get_module_base_by_hash(0x40054168);
        if !crt_base.is_null() {
            let exports = [0x10f905d5, 0x54693fbe, 0x5cc35eb6, 0x2971e0e0];
            for &exp_hash in exports.iter() {
                if let Some(func_ptr) = get_export_by_hash(crt_base, exp_hash) {
                    match exp_hash {
                        0x10f905d5 | 0x54693fbe => {
                            let get_ptr: extern "C" fn() -> *mut *mut c_void = transmute(func_ptr);
                            let ptr_to_str = get_ptr();
                            if !ptr_to_str.is_null() {
                                let replacement = if exp_hash == 0x10f905d5 {
                                    new_u16
                                } else {
                                    new_a
                                };
                                *ptr_to_str = replacement;
                            }
                        }
                        0x5cc35eb6 | 0x2971e0e0 => {
                            let global_ptr = func_ptr as *mut *mut c_void;
                            let replacement = if exp_hash == 0x5cc35eb6 {
                                new_u16
                            } else {
                                new_a
                            };
                            let mut old_prot = 0u32;
                            let mut p_addr = global_ptr as *mut c_void;
                            let mut p_size = 8usize;
                            nt_protect(
                                -1isize as HANDLE,
                                &mut p_addr,
                                &mut p_size,
                                PAGE_READWRITE,
                                &mut old_prot,
                            );
                            *global_ptr = replacement;
                            nt_protect(
                                -1isize as HANDLE,
                                &mut p_addr,
                                &mut p_size,
                                old_prot,
                                &mut old_prot,
                            );
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok((new_u16 as *mut u16, new_a as *mut i8))
    }
}

fn apply(
    target: *mut u8,
    return_val: *mut c_void,
    nt_protect: extern "system" fn(HANDLE, *mut *mut c_void, *mut usize, u32, *mut u32) -> i32,
) {
    unsafe {
        let mut old_prot = 0u32;
        let mut p_addr = target as *mut c_void;
        let mut p_size = 13usize;
        nt_protect(
            -1isize as HANDLE,
            &mut p_addr,
            &mut p_size,
            PAGE_EXECUTE_READWRITE,
            &mut old_prot,
        );
        let mut patch_code = [
            0x48, 0xB8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC3,
        ];
        std::ptr::copy_nonoverlapping(
            &return_val as *const _ as *const u8,
            patch_code.as_mut_ptr().add(2),
            8,
        );
        std::ptr::copy_nonoverlapping(patch_code.as_ptr(), target, patch_code.len());
        nt_protect(
            -1isize as HANDLE,
            &mut p_addr,
            &mut p_size,
            old_prot,
            &mut old_prot,
        );
    }
}
