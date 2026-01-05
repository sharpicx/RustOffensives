use crate::get_date;
use chrono::{DateTime, Local};
use colored::*;
use std::fs;
use std::path::Path;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::SystemTime;

#[derive(Debug)]
struct Entry {
    path: String,
    created: SystemTime,
}

fn fast_collect_stream(
    root: &Path,
    tx: &Sender<Entry>,
    folder: &Option<String>,
    exclude_exts: &[String],
    exclude_roots: &[String],
    exclude_dirs: &[String],
) {
    let mut stack = vec![root.to_path_buf()];

    while let Some(dir) = stack.pop() {
        let dl = dir.to_string_lossy().to_lowercase();

        if exclude_roots.iter().any(|x| dl.starts_with(x)) {
            continue;
        }

        match folder {
            Some(fd) if !dl.contains(fd) => continue,
            _ => {}
        }

        let rd = match fs::read_dir(&dir) {
            Ok(v) => v,
            Err(_) => continue,
        };

        for e in rd.flatten() {
            let p = e.path();

            if let Some(ext) = p.extension().and_then(|x| x.to_str()) {
                let ext = ext.to_lowercase();
                if exclude_exts.iter().any(|x| x == &ext) {
                    continue;
                }
            }

            let meta = match e.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };

            if meta.is_dir() {
                let dir_name = p
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_lowercase())
                    .unwrap_or_default();

                if exclude_dirs.iter().any(|x| dir_name.contains(x)) {
                    continue;
                }

                stack.push(p);
                continue;
            }

            let ct = match meta.created() {
                Ok(t) => t,
                Err(_) => continue,
            };

            let _ = tx.send(Entry {
                path: p.display().to_string(),
                created: ct,
            });
        }
    }
}

fn fmt_time(t: SystemTime) -> String {
    let dt: DateTime<Local> = t.into();
    dt.format("%d-%m-%Y %H:%M:%S").to_string()
}

pub fn enum_sus_files_dirs(
    filter: Option<String>,
    folder: Option<String>,
    add_roots: Vec<String>,
    exclude_roots: Option<String>,
    exclude_exts: Option<String>,
    exclude_date: Option<String>,
    exclude_dirs: Option<String>,
) {
    let mut roots: Vec<String> = vec![
        r"C:\Windows".into(),
        r"C:\ProgramData".into(),
        r"C:\Users".into(),
    ];
    roots.extend(add_roots);

    let exclude_dirs: Vec<String> = exclude_dirs
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();

    let exclude_exts: Vec<String> = exclude_exts
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();

    let exclude_list: Vec<String> = exclude_roots
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();

    roots.retain(|r| {
        let rl = r.to_lowercase();
        !exclude_list.iter().any(|x| rl.starts_with(x))
    });

    let folder_l = folder.clone().map(|x| x.to_lowercase());
    let (tx, rx) = std::sync::mpsc::channel();

    for r in roots {
        let tx = tx.clone();
        let folder_l = folder_l.clone();
        let exclude_exts = exclude_exts.clone();
        let exclude_list = exclude_list.clone();
        let exclude_dirs = exclude_dirs.clone();

        thread::spawn(move || {
            fast_collect_stream(
                Path::new(&r),
                &tx,
                &folder_l,
                &exclude_exts,
                &exclude_list,
                &exclude_dirs,
            );
        });
    }

    drop(tx);

    let filter = filter.map(|x| x.to_lowercase());
    let folder = folder.map(|x| x.to_lowercase());

    let exclude_dates: Vec<get_date::DateFilter> = exclude_date
        .unwrap_or_default()
        .split(',')
        .filter_map(|s| get_date::parse_filter(s.trim()))
        .collect();

    let mut printed_header = false;

    for e in rx {
        let t = fmt_time(e.created);
        let tl = t.to_lowercase();
        let pl = e.path.to_lowercase();
        let dt: DateTime<Local> = e.created.into();

        let filter_failed = filter.as_ref().map_or(false, |f| !tl.contains(f));
        let folder_failed = folder.as_ref().map_or(false, |fd| !pl.contains(fd));
        let exclude_failed = exclude_dates
            .iter()
            .any(|f| get_date::matches_filter(&dt, f));

        if filter_failed || folder_failed || exclude_failed {
            continue;
        }

        if !printed_header {
            println!("{}", "[+] Found Files or Directories:".green().bold());
            printed_header = true;
        }

        println!("    {} {}  {}", "[+]".yellow(), t.blue(), e.path.white());
    }
}
