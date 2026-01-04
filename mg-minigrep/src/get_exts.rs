use colored::*;
use std::collections::HashMap;
use std::ffi::OsString;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::path::PathBuf;
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::Storage::FileSystem::*;

fn split_csv(raw: String) -> Vec<String> {
    raw.split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect()
}

fn match_path_or_dir(path: &PathBuf, exclude_list: &[String]) -> bool {
    let p_str = path.to_string_lossy().to_lowercase();
    exclude_list.iter().any(|ex| p_str.contains(ex))
}

pub fn run_get_exts(roots_raw: String, exclude_roots_raw: String) {
    let roots = split_csv(roots_raw);
    let exclude_roots = split_csv(exclude_roots_raw);

    // Default system exclusion untuk kestabilan dan kecepatan
    let system_excludes = vec![
        "system volume information".to_string(),
        "$recycle.bin".to_string(),
        "windows\\winsxs".to_string(),
    ];

    let mut stats: HashMap<String, (usize, Vec<PathBuf>)> = HashMap::new();
    let mut stack: Vec<PathBuf> = roots
        .into_iter()
        .map(PathBuf::from)
        .filter(|r| !match_path_or_dir(r, &exclude_roots))
        .collect();

    println!(
        "{} {}",
        "[*]".bright_blue(),
        "Enumerating drive... please wait.".bright_white()
    );

    while let Some(root) = stack.pop() {
        let mut pat = root.clone();
        pat.push("*");
        let pat_w: Vec<u16> = pat.as_os_str().encode_wide().chain([0]).collect();

        let mut data = WIN32_FIND_DATAW::default();
        let h = unsafe { FindFirstFileW(pat_w.as_ptr(), &mut data) };
        if h == INVALID_HANDLE_VALUE {
            continue;
        }

        loop {
            let len = data.cFileName.iter().position(|&c| c == 0).unwrap_or(260);
            let name = OsString::from_wide(&data.cFileName[..len]);

            if name != "." && name != ".." {
                let mut full = root.clone();
                full.push(&name);

                // Filter exclude-roots dan system excludes
                if match_path_or_dir(&full, &exclude_roots)
                    || match_path_or_dir(&full, &system_excludes)
                {
                    if unsafe { FindNextFileW(h, &mut data) } == 0 {
                        break;
                    }
                    continue;
                }

                let is_dir = (data.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0;

                if is_dir {
                    stack.push(full);
                } else {
                    let ext = full
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("no-ext")
                        .to_lowercase();

                    let entry = stats.entry(ext).or_insert((0, Vec::new()));
                    entry.0 += 1;

                    // Batasi hanya simpan 5 contoh path agar RAM tidak jebol di C:\
                    if entry.1.len() < 5 {
                        entry.1.push(full);
                    }
                }
            }

            if unsafe { FindNextFileW(h, &mut data) } == 0 {
                break;
            }
        }
        unsafe { FindClose(h) };
    }

    print_report(stats);
}

fn print_report(stats: HashMap<String, (usize, Vec<PathBuf>)>) {
    let mut final_list: Vec<_> = stats.into_iter().collect();

    // Sort berdasarkan jumlah file terbanyak
    final_list.sort_by(|a, b| b.1.0.cmp(&a.1.0));

    println!(
        "\n{}",
        "===================================================".bright_black()
    );
    println!(
        "{}",
        "         FILES ENUMERATION REPORT                  "
            .bright_cyan()
            .bold()
    );
    println!(
        "{}\n",
        "===================================================".bright_black()
    );

    for (ext, (count, examples)) in &final_list {
        if *count == 0 {
            continue;
        }

        println!(
            "{} {} ({} files)",
            "[+]".bright_green(),
            ext.to_uppercase().bold().yellow(),
            count.to_string().bright_white()
        );

        for ex in examples {
            println!(
                "    {} {}",
                "->".bright_black(),
                ex.display().to_string().white()
            );
        }

        println!();
    }

    println!("{}", "--- QUICK SUMMARY ---".bright_cyan().bold());
    let mut total_files = 0;
    for (ext, (count, _)) in final_list {
        total_files += count;
        if count > 0 {
            println!("{:<12} : {}", ext.to_uppercase().yellow(), count);
        }
    }

    println!(
        "\n{}: {}",
        "TOTAL FILES SCANNED".bold(),
        total_files.to_string().bright_green()
    );
}
