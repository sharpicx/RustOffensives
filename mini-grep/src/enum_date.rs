use chrono::{DateTime, Datelike, Local};
use colored::*;
use std::collections::BTreeSet;
use std::ffi::OsString;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::path::PathBuf;
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::SystemTime;
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::Storage::FileSystem::*;

#[derive(Clone, Copy)]
pub enum DateFilter {
    Year(u32),
    Day(u32),
    DayMonth(u32, u32),
    DayMonthYear(u32, u32, u32),
}

pub fn enum_date(drives: String, filter_date: String) {
    let roots: Vec<PathBuf> = drives
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .collect();

    let filters: Vec<DateFilter> = filter_date
        .split(',')
        .filter_map(|s| parse_filter(s.trim()))
        .collect();

    let (tx, rx) = mpsc::channel::<(DateTime<Local>, PathBuf)>();

    let filters_clone = filters.clone();
    thread::spawn(move || {
        let mut seen_dates: BTreeSet<(u32, u32, u32)> = BTreeSet::new();
        for root in roots {
            traverse_channel(&root, &filters_clone, &mut seen_dates, &tx);
        }
    });

    for (dt, _path) in rx {
        println!(
            "[{}] {}",
            "+".bright_red(),
            dt.format("%d-%m-%Y - %I:%M %p"),
        );
    }
}

fn traverse_channel(
    path: &PathBuf,
    filters: &Vec<DateFilter>,
    seen: &mut BTreeSet<(u32, u32, u32)>,
    tx: &Sender<(DateTime<Local>, PathBuf)>,
) {
    let mut stack: Vec<PathBuf> = Vec::new();
    stack.push(path.clone());

    while let Some(current) = stack.pop() {
        let mut pat = current.clone();
        pat.push("*");
        let pat_w: Vec<u16> = pat.as_os_str().encode_wide().chain([0]).collect();

        let mut data = WIN32_FIND_DATAW::default();
        let h = unsafe { FindFirstFileW(pat_w.as_ptr(), &mut data) };
        if h == INVALID_HANDLE_VALUE {
            continue;
        }

        loop {
            let len = data
                .cFileName
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(data.cFileName.len());
            let name = OsString::from_wide(&data.cFileName[..len]);
            if name != "." && name != ".." {
                let mut full = current.clone();
                full.push(&name);

                let attrs = data.dwFileAttributes;
                let is_dir = (attrs & FILE_ATTRIBUTE_DIRECTORY) != 0;
                let st = filetime_to_systemtime(&data.ftCreationTime);
                let dt: DateTime<Local> = st.into();

                if filters.is_empty() || !filters.iter().any(|f| matches_filter(&dt, f)) {
                    let date_key = (dt.day(), dt.month(), dt.year() as u32);
                    if seen.insert(date_key) {
                        let _ = tx.send((dt, full.clone()));
                    }
                }

                if is_dir {
                    stack.push(full);
                }
            }

            if unsafe { FindNextFileW(h, &mut data) } == 0 {
                break;
            }
        }

        unsafe { FindClose(h) };
    }
}

fn filetime_to_systemtime(ft: &FILETIME) -> SystemTime {
    use std::time::{Duration, UNIX_EPOCH};
    let intervals = ((ft.dwHighDateTime as u64) << 32) | ft.dwLowDateTime as u64;
    let secs_since_1601 = intervals / 10_000_000;
    let nanos_remainder = (intervals % 10_000_000) * 100;
    let secs_since_unix = secs_since_1601 - 11644473600;
    UNIX_EPOCH + Duration::new(secs_since_unix, nanos_remainder as u32)
}

pub fn parse_filter(s: &str) -> Option<DateFilter> {
    let parts: Vec<_> = s.split('-').collect();
    match parts.len() {
        1 => {
            let n = parts[0].parse::<u32>().ok()?;
            if n > 31 {
                Some(DateFilter::Year(n))
            } else {
                Some(DateFilter::Day(n))
            }
        }
        2 => {
            let day = parts[0].parse::<u32>().ok()?;
            let month = parts[1].parse::<u32>().ok()?;
            Some(DateFilter::DayMonth(day, month))
        }
        3 => {
            let day = parts[0].parse::<u32>().ok()?;
            let month = parts[1].parse::<u32>().ok()?;
            let year = parts[2].parse::<u32>().ok()?;
            Some(DateFilter::DayMonthYear(day, month, year))
        }
        _ => None,
    }
}

pub fn matches_filter(dt: &DateTime<Local>, filter: &DateFilter) -> bool {
    match filter {
        DateFilter::Year(y) => dt.year() as u32 == *y,
        DateFilter::Day(d) => dt.day() == *d,
        DateFilter::DayMonth(d, m) => dt.day() == *d && dt.month() == *m,
        DateFilter::DayMonthYear(d, m, y) => {
            dt.day() == *d && dt.month() == *m && dt.year() as u32 == *y
        }
    }
}
