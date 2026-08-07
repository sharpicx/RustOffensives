use crc::{CRC_32_ISCSI, Crc};
use std::env;
use std::ptr;
use winapi::shared::minwindef::{DWORD, HMODULE};
use winapi::um::handleapi::CloseHandle;
use winapi::um::processthreadsapi::{
    CreateProcessW, OpenProcess, PROCESS_INFORMATION, ResumeThread, STARTUPINFOW,
};
use winapi::um::psapi::{EnumProcessModulesEx, GetModuleBaseNameW, LIST_MODULES_ALL};
use winapi::um::winbase::CREATE_SUSPENDED;
use winapi::um::winnt::{IMAGE_DOS_HEADER, IMAGE_EXPORT_DIRECTORY, IMAGE_NT_HEADERS64};

const CRC32C: Crc<u32> = Crc::<u32>::new(&CRC_32_ISCSI);
const IMAGE_NT_SIGNATURE: u32 = 0x0000_4550;
const IMAGE_DOS_SIGNATURE: u16 = 0x5A4D;
const IMAGE_DIRECTORY_ENTRY_EXPORT: usize = 0;

fn compute_hash(s: &str, upper: bool, include_null: bool) -> u32 {
    let mut digest = CRC32C.digest();
    let utf16_chars: Vec<u16> = s.encode_utf16().collect();
    for &c in &utf16_chars {
        let mut b = (c & 0xFF) as u8;
        if upper {
            b = b.to_ascii_uppercase();
        } else {
            b = b.to_ascii_lowercase();
        }
        digest.update(&[b]);
    }
    if include_null {
        digest.update(&[0]);
    }
    digest.finalize()
}

fn dump_remote_modules(process: winapi::um::winnt::HANDLE) {
    unsafe {
        let mut h_mods: [HMODULE; 1024] = core::mem::zeroed();
        let mut cb_needed: DWORD = 0;

        let mut retries = 0;
        let mut count = 0;
        while retries < 40 {
            if EnumProcessModulesEx(
                process,
                h_mods.as_mut_ptr(),
                core::mem::size_of_val(&h_mods) as DWORD,
                &mut cb_needed,
                LIST_MODULES_ALL,
            ) != 0
            {
                count = (cb_needed as usize) / core::mem::size_of::<HMODULE>();
                if count > 0 {
                    break;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
            retries += 1;
        }

        println!("\n[+] --- DUMP MODULES (Total: {}) ---", count);
        for i in 0..count {
            let mut mod_name: [u16; 260] = core::mem::zeroed();
            if GetModuleBaseNameW(
                process,
                h_mods[i],
                mod_name.as_mut_ptr(),
                core::mem::size_of_val(&mod_name) as DWORD,
            ) != 0
            {
                let len = mod_name
                    .iter()
                    .position(|&c| c == 0)
                    .unwrap_or(mod_name.len());
                let name = String::from_utf16_lossy(&mod_name[..len]);
                let mod_base = h_mods[i] as usize;

                let h1 = compute_hash(&name, false, true);
                let h2 = compute_hash(&name, true, true);
                let h3 = compute_hash(&name, false, false);
                let h4 = compute_hash(&name, true, false);

                println!(
                    "Module: {:<25} | Base: 0x{:016X} | h1: {:08X} | h2: {:08X} | h3: {:08X} | h4: {:08X}",
                    name, mod_base, h1, h2, h3, h4
                );
            }
        }
    }
}

fn dump_remote_exports(process: winapi::um::winnt::HANDLE, module_base: usize, module_name: &str) {
    unsafe {
        let mut dos_header: IMAGE_DOS_HEADER = core::mem::zeroed();
        let mut bytes_read: usize = 0;

        winapi::um::memoryapi::ReadProcessMemory(
            process,
            module_base as *const _,
            &mut dos_header as *mut _ as *mut _,
            core::mem::size_of::<IMAGE_DOS_HEADER>(),
            &mut bytes_read,
        );

        if dos_header.e_magic != IMAGE_DOS_SIGNATURE {
            return;
        }

        let mut nt_headers: IMAGE_NT_HEADERS64 = core::mem::zeroed();
        winapi::um::memoryapi::ReadProcessMemory(
            process,
            (module_base + dos_header.e_lfanew as usize) as *const _,
            &mut nt_headers as *mut _ as *mut _,
            core::mem::size_of::<IMAGE_NT_HEADERS64>(),
            &mut bytes_read,
        );

        if nt_headers.Signature != IMAGE_NT_SIGNATURE {
            return;
        }

        let export_dir_rva = nt_headers.OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_EXPORT]
            .VirtualAddress as usize;
        if export_dir_rva == 0 {
            return;
        }

        let export_dir_addr = module_base + export_dir_rva;
        let mut export_dir: IMAGE_EXPORT_DIRECTORY = core::mem::zeroed();
        winapi::um::memoryapi::ReadProcessMemory(
            process,
            export_dir_addr as *const _,
            &mut export_dir as *mut _ as *mut _,
            core::mem::size_of::<IMAGE_EXPORT_DIRECTORY>(),
            &mut bytes_read,
        );

        let mut names: Vec<u32> = vec![0; export_dir.NumberOfNames as usize];
        winapi::um::memoryapi::ReadProcessMemory(
            process,
            (module_base + export_dir.AddressOfNames as usize) as *const _,
            names.as_mut_ptr() as *mut _,
            names.len() * core::mem::size_of::<u32>(),
            &mut bytes_read,
        );

        let mut ordinals: Vec<u16> = vec![0; export_dir.NumberOfNames as usize];
        winapi::um::memoryapi::ReadProcessMemory(
            process,
            (module_base + export_dir.AddressOfNameOrdinals as usize) as *const _,
            ordinals.as_mut_ptr() as *mut _,
            ordinals.len() * core::mem::size_of::<u16>(),
            &mut bytes_read,
        );

        let mut functions: Vec<u32> = vec![0; export_dir.NumberOfFunctions as usize];
        winapi::um::memoryapi::ReadProcessMemory(
            process,
            (module_base + export_dir.AddressOfFunctions as usize) as *const _,
            functions.as_mut_ptr() as *mut _,
            functions.len() * core::mem::size_of::<u32>(),
            &mut bytes_read,
        );

        println!("\n[+] Dump exports for: {}", module_name);
        for i in 0..names.len() {
            let name_addr = module_base + names[i] as usize;
            let mut name_bytes = Vec::new();
            let mut buf: [u8; 1] = [0];
            let mut offset = 0;

            loop {
                winapi::um::memoryapi::ReadProcessMemory(
                    process,
                    (name_addr + offset) as *const _,
                    buf.as_mut_ptr() as *mut _,
                    1,
                    &mut bytes_read,
                );
                if bytes_read == 0 || buf[0] == 0 {
                    break;
                }
                name_bytes.push(buf[0]);
                offset += 1;
            }

            let name_str = core::str::from_utf8(&name_bytes).unwrap_or("UNKNOWN");

            let h1 = CRC32C.checksum(&name_bytes);
            let mut digest_null = CRC32C.digest();
            digest_null.update(&name_bytes);
            digest_null.update(&[0]);
            let h2 = digest_null.finalize();

            let upper_bytes: Vec<u8> = name_bytes.iter().map(|b| b.to_ascii_uppercase()).collect();
            let h3 = CRC32C.checksum(&upper_bytes);

            let mut digest_upper_null = CRC32C.digest();
            digest_upper_null.update(&upper_bytes);
            digest_upper_null.update(&[0]);
            let h4 = digest_upper_null.finalize();

            let ordinal = ordinals[i] as usize;
            if let Some(&func_rva) = functions.get(ordinal) {
                let func_addr = module_base + func_rva as usize;
                println!(
                    "API: {:<30} | Addr: 0x{:016X} | h1: {:08X} | h2: {:08X} | h3: {:08X} | h4: {:08X}",
                    name_str, func_addr, h1, h2, h3, h4
                );
            }
        }
    }
}

fn print_usage() {
    println!("Usage:");
    println!("  program.exe --pid <PID> --dump-modules");
    println!("  program.exe --pid <PID> --dump-exports <MODULE_NAME>");
    println!("  program.exe --spawn <PATH> --dump-modules");
    println!("  program.exe --spawn <PATH> --dump-exports <MODULE_NAME>");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        print_usage();
        return;
    }

    let mut target_pid: DWORD = 0;
    let mut spawn_path: Option<String> = None;
    let mut dump_modules_flag = false;
    let mut dump_exports_target: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--pid" => {
                if i + 1 < args.len() {
                    target_pid = args[i + 1].parse().unwrap_or(0);
                    i += 1;
                }
            }
            "--spawn" => {
                if i + 1 < args.len() {
                    spawn_path = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--dump-modules" => {
                dump_modules_flag = true;
            }
            "--dump-exports" => {
                if i + 1 < args.len() {
                    dump_exports_target = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    unsafe {
        let process;
        let mut pi: Option<PROCESS_INFORMATION> = None;

        if let Some(path) = spawn_path {
            let mut wide_path: Vec<u16> = path.encode_utf16().collect();
            wide_path.push(0);

            let mut si: STARTUPINFOW = core::mem::zeroed();
            si.cb = core::mem::size_of::<STARTUPINFOW>() as DWORD;
            let mut proc_info: PROCESS_INFORMATION = core::mem::zeroed();

            let success = CreateProcessW(
                ptr::null(),
                wide_path.as_mut_ptr(),
                ptr::null_mut(),
                ptr::null_mut(),
                0,
                CREATE_SUSPENDED,
                ptr::null_mut(),
                ptr::null(),
                &mut si,
                &mut proc_info,
            );

            if success == 0 {
                println!("[-] Failed to create process in suspended state.");
                return;
            }

            println!("[+] Successfully spawned suspended process!");
            println!("[+] PID : {}", proc_info.dwProcessId);
            println!("[+] TID : {}", proc_info.dwThreadId);

            process = proc_info.hProcess;
            pi = Some(proc_info);
        } else if target_pid != 0 {
            process = OpenProcess(winapi::um::winnt::PROCESS_ALL_ACCESS, 0, target_pid);
            if process.is_null() {
                println!("[-] Failed to open process with PID: {}", target_pid);
                return;
            }
        } else {
            println!("[-] Neither --pid nor --spawn provided.");
            print_usage();
            return;
        }

        if let Some(ref proc_info) = pi {
            ResumeThread(proc_info.hThread);
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        if dump_modules_flag {
            dump_remote_modules(process);
        }

        if let Some(mod_name) = &dump_exports_target {
            let mut h_mods: [HMODULE; 1024] = core::mem::zeroed();
            let mut cb_needed: DWORD = 0;
            let mut found_base: Option<usize> = None;

            let mut retries = 0;
            while retries < 40 {
                if EnumProcessModulesEx(
                    process,
                    h_mods.as_mut_ptr(),
                    core::mem::size_of_val(&h_mods) as DWORD,
                    &mut cb_needed,
                    LIST_MODULES_ALL,
                ) != 0
                {
                    let count = (cb_needed as usize) / core::mem::size_of::<HMODULE>();
                    if count > 0 {
                        for j in 0..count {
                            let mut name_buf: [u16; 260] = core::mem::zeroed();
                            if GetModuleBaseNameW(
                                process,
                                h_mods[j],
                                name_buf.as_mut_ptr(),
                                core::mem::size_of_val(&name_buf) as DWORD,
                            ) != 0
                            {
                                let len = name_buf
                                    .iter()
                                    .position(|&c| c == 0)
                                    .unwrap_or(name_buf.len());
                                let current_name = String::from_utf16_lossy(&name_buf[..len]);
                                if current_name.eq_ignore_ascii_case(mod_name) {
                                    found_base = Some(h_mods[j] as usize);
                                    break;
                                }
                            }
                        }
                        if found_base.is_some() {
                            break;
                        }
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
                retries += 1;
            }

            if let Some(base) = found_base {
                dump_remote_exports(process, base, mod_name);
            } else {
                println!("[-] Module {} not found in target process.", mod_name);
            }
        }

        if let Some(proc_info) = pi {
            CloseHandle(proc_info.hThread);
        }

        CloseHandle(process);
    }
}
