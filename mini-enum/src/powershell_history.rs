use regex::Regex;
use std::path::Path;
use std::path::PathBuf;
use std::{fs, usize};

struct DirectoryListDTO {
    path: String,
}

struct DirectoryQuery {
    path: PathBuf,
    depth: usize,
}

struct PowerShellHistoryDTO {
    user_name: String,
    history_path: String,
    matched: String,
    context: String,
}

fn scan_powershell_history(
    dirs: &[DirectoryListDTO],
    with_regex: bool,
    regex: Option<&[Regex]>,
    context: usize,
) -> Vec<PowerShellHistoryDTO> {
    let mut results = Vec::new();
    for dir in dirs {
        let dir_path = dir.path.trim_end_matches('\\');
        let parts: Vec<&str> = dir_path.split('\\').collect();
        let user_name = match parts.last() {
            Some(v) => v.to_string(),
            None => continue,
        };
        if matches!(
            user_name.as_str(),
            "Public" | "Default" | "Default User" | "All Users"
        ) {
            continue;
        }
        let history_path = format!(
            "{}\\AppData\\Roaming\\Microsoft\\Windows\\PowerShell\\PSReadline\\ConsoleHost_history.txt",
            dir_path
        );
        if !Path::new(&history_path).exists() {
            continue;
        }
        let content = match fs::read_to_string(&history_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let lines: Vec<&str> = content.lines().collect();
        let mut context_lines: Vec<String> = Vec::new();
        if with_regex {
            let regexes = match regex {
                Some(r) => r,
                None => return Vec::new(),
            };
            for reg in regexes {
                let m = match reg.find(&content) {
                    Some(m) => m,
                    None => continue,
                };
                for (i, line) in lines.iter().enumerate() {
                    if !line.contains(m.as_str()) {
                        continue;
                    }
                    let mut printed = 0;
                    let mut j = 1;
                    while i >= j && printed < context {
                        let l = lines[i - j].trim();
                        if !l.is_empty() {
                            context_lines.push(l.to_string());
                            printed += 1;
                        }
                        j += 1;
                    }
                    context_lines.push(m.as_str().trim().to_string());
                    printed = 0;
                    j = 1;
                    while i + j < lines.len() && printed < context {
                        let l = lines[i + j].trim();
                        if !l.is_empty() {
                            context_lines.push(l.to_string());
                            printed += 1;
                        }
                        j += 1;
                    }
                    break;
                }
                results.push(PowerShellHistoryDTO {
                    user_name: user_name.clone(),
                    history_path: history_path.clone(),
                    matched: m.as_str().to_string(),
                    context: context_lines.join("\n"),
                });
            }
        } else {
            for (i, line) in lines.iter().enumerate() {
                let line_trimmed = line.trim();
                if line_trimmed.is_empty() {
                    continue;
                }
                let mut context_lines = Vec::new();
                let mut printed = 0;
                let mut j = 1;
                while i >= j && printed < context {
                    let l = lines[i - j].trim();
                    if !l.is_empty() {
                        context_lines.push(l.to_string());
                        printed += 1;
                    }
                    j += 1;
                }
                context_lines.push(line_trimmed.to_string());
                printed = 0;
                j = 1;
                while i + j < lines.len() && printed < context {
                    let l = lines[i + j].trim();
                    if !l.is_empty() {
                        context_lines.push(l.to_string());
                        printed += 1;
                    }
                    j += 1;
                }
                results.push(PowerShellHistoryDTO {
                    user_name: user_name.clone(),
                    history_path: history_path.clone(),
                    matched: line_trimmed.to_string(),
                    context: context_lines.join("\n"),
                });
            }
        }
    }
    results
}

fn get_process_cmdline_regex() -> Vec<Regex> {
    vec![
        Regex::new(r"(ConvertTo-SecureString.*AsPlainText.*)").unwrap(),
        Regex::new(r"(net(.exe)?.*user .*)").unwrap(),
        Regex::new(r"(net(.exe)?.*use .*)").unwrap(),
        Regex::new(r"(cmdkey(.exe)?.*/pass:.*)").unwrap(),
        Regex::new(r"(ssh(.exe)?.*-i .*)").unwrap(),
        Regex::new(r"(psexec(.exe)?.*-p .*)").unwrap(),
        Regex::new(r"(psexec64(.exe)?.*-p .*)").unwrap(),
        Regex::new(r"(winrm(.vbs)?.*-p .*)").unwrap(),
        Regex::new(r"(winrs(.exe)?.*/p(assword)? .*)").unwrap(),
        Regex::new(r"(putty(.exe)?.*-pw .*)").unwrap(),
        Regex::new(r"(pscp(.exe)?.*-pw .*)").unwrap(),
        Regex::new(r"(kitty(.exe)?.*(-pw|-pass) .*)").unwrap(),
        Regex::new(r"(bitsadmin(.exe)?.*(/RemoveCredentials|/SetCredentials) .*)").unwrap(),
        Regex::new(r"(bootcfg(.exe)?.*/p .*)").unwrap(),
        Regex::new(r"(certreq(.exe)?.*-p .*)").unwrap(),
        Regex::new(r"(certutil(.exe)?.*-p .*)").unwrap(),
        Regex::new(r"(driverquery(.exe)?.*/p .*)").unwrap(),
        Regex::new(r"(eventcreate(.exe)?.*/p .*)").unwrap(),
        Regex::new(r"(getmac(.exe)?.*/p .*)").unwrap(),
        Regex::new(r"(gpfixup(.exe)?.*/pwd:.*)").unwrap(),
        Regex::new(r"(gpresult(.exe)?.*/p .*)").unwrap(),
        Regex::new(r"(mapadmin(.exe)?.*-p .*)").unwrap(),
        Regex::new(r"(mount(.exe)?.*-p:.*)").unwrap(),
        Regex::new(r"(nfsadmin(.exe)?.*-p .*)").unwrap(),
        Regex::new(r"(openfiles(.exe)?.*/p .*)").unwrap(),
        Regex::new(r"(cscript.*-w .*)").unwrap(),
        Regex::new(r"(schtasks(.exe)?.*(/p|/rp) .*)").unwrap(),
        Regex::new(r"(setx(.exe)?.*/p .*)").unwrap(),
        Regex::new(r"(systeminfo(.exe)?.*/p .*)").unwrap(),
        Regex::new(r"(takeown(.exe)?.*/p .*)").unwrap(),
        Regex::new(r"(taskkill(.exe)?.*/p .*)").unwrap(),
        Regex::new(r"(tscon(.exe)?.*/password:.*)").unwrap(),
        Regex::new(r"(wecutil(.exe)?.*(/up|/cup|/p):.*)").unwrap(),
        Regex::new(r"(wmic(.exe)?.*/password:.*)").unwrap(),
    ]
}

fn get_directories(
    regex: Option<&Regex>,
    ignore_errors: bool,
    query: &DirectoryQuery,
    max_depth: usize,
    dir_stack: &mut Vec<DirectoryQuery>,
) -> Vec<DirectoryListDTO> {
    let mut result = Vec::new();
    let entries = match fs::read_dir(&query.path) {
        Ok(e) => e,
        Err(err) => {
            if !ignore_errors {
                eprintln!("{}", err);
            }
            return result;
        }
    };
    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(err) => {
                if !ignore_errors {
                    eprintln!("{}", err);
                }
                continue;
            }
        };
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let path_str = path.to_string_lossy().to_string();
        if query.depth + 1 <= max_depth {
            dir_stack.push(DirectoryQuery {
                path: path.clone(),
                depth: query.depth + 1,
            });
        }
        let matches = match regex {
            Some(r) => r.is_match(&path_str),
            None => true,
        };
        if !matches {
            continue;
        }
        let normalized = if path_str.ends_with('\\') {
            path_str
        } else {
            format!("{}\\", path_str)
        };
        result.push(DirectoryListDTO { path: normalized });
    }
    result
}

pub fn enum_powershell_histories(use_regex: bool) {
    let query = DirectoryQuery {
        path: PathBuf::from(r"C:\Users"),
        depth: 0,
    };
    let mut stack: Vec<DirectoryQuery> = Vec::new();
    let dirs = get_directories(None, true, &query, usize::MAX, &mut stack);
    let context = 3;
    let findings = if use_regex {
        let regexes = get_process_cmdline_regex();
        scan_powershell_history(&dirs, true, Some(&regexes), context)
    } else {
        scan_powershell_history(&dirs, false, None, context)
    };
    for f in findings {
        println!("User    : {}", f.user_name);
        println!("Path    : {}", f.history_path);
        println!("Matched : {}", f.matched);
        println!("Context :\n{}\n", f.context);
        println!("----------------------------------------");
    }
}
