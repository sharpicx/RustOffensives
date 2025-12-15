use chrono::Local;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Memory::GlobalLock;
use windows::Win32::System::Memory::GlobalUnlock;
use windows::{
    Win32::Foundation::*, Win32::System::DataExchange::*, Win32::System::Ole::CF_HDROP,
    Win32::UI::Shell::*, Win32::UI::WindowsAndMessaging::*, core::*,
};

fn clipboard_text_logging(text: &str) {
    let folder = r"C:\Windows\Temp\EFEW\Clip.CF_UNICODETEXT";
    fs::create_dir_all(folder).ok();

    let timestamp = Local::now().format("[%d/%m/%Y - %-I:%M %p]").to_string();
    let log_path = Path::new(folder).join("clip.log");

    let mut file = File::options()
        .append(true)
        .create(true)
        .open(log_path)
        .unwrap();
    writeln!(file, "{} {}", timestamp, text).ok();
}

fn clipboard_files_logging(hdrop: HDROP) {
    unsafe {
        let folder = r"C:\Windows\Temp\EFEW\Clip.CF_HDROP";
        fs::create_dir_all(folder).ok();

        let count = DragQueryFileW(hdrop, 0xFFFFFFFF, None);
        for i in 0..count {
            let mut buf = vec![0u16; 260];
            DragQueryFileW(hdrop, i, Some(&mut buf));
            let path = String::from_utf16_lossy(&buf)
                .trim_end_matches('\0')
                .to_string();

            if let Some(filename) = Path::new(&path).file_name() {
                let timestamp = Local::now().format("%d-%m-%Y_%-I-%M-%p").to_string();
                let new_name = format!("{}_{}.bak", filename.to_string_lossy(), timestamp);
                let dest_path = Path::new(folder).join(new_name);
                fs::copy(&path, dest_path).ok();
            }
        }
    }
}

const CF_UNICODETEXT: u32 = 13;

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_CLIPBOARDUPDATE => unsafe {
            if OpenClipboard(Some(hwnd)).is_ok() {
                if let Ok(h) = GetClipboardData(CF_UNICODETEXT) {
                    let hglobal = HGLOBAL(h.0);
                    let ptr = GlobalLock(hglobal) as *const u16;
                    if !ptr.is_null() {
                        let mut len = 0;
                        while *ptr.add(len) != 0 {
                            len += 1;
                        }
                        let text = String::from_utf16_lossy(std::slice::from_raw_parts(ptr, len));
                        println!("[Text] {}", text);
                        clipboard_text_logging(&text);
                        let _ = GlobalUnlock(hglobal);
                    }
                }
                if let Ok(h) = GetClipboardData(CF_HDROP.0 as u32) {
                    let hdrop = HDROP(h.0);
                    let count = DragQueryFileW(hdrop, 0xFFFFFFFF, None);
                    for i in 0..count {
                        let mut buf = vec![0u16; 260];
                        DragQueryFileW(hdrop, i, Some(&mut buf));
                        let path = String::from_utf16_lossy(&buf);
                        println!("File: {}", path.trim_end_matches('\0'));
                    }
                    clipboard_files_logging(hdrop);
                }

                let _ = CloseClipboard();
            }
            LRESULT(0)
        },
        WM_DESTROY => unsafe {
            PostQuitMessage(0);
            LRESULT(0)
        },
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

unsafe fn run_clipboard_listener() -> Result<()> {
    unsafe {
        let hinstance = GetModuleHandleW(None)?;
        let class_name = w!("ClipboardListener");
        let hinstance = HINSTANCE(hinstance.0);

        let wc = WNDCLASSW {
            lpfnWndProc: Some(wndproc),
            hInstance: hinstance,
            lpszClassName: class_name,
            ..Default::default()
        };

        RegisterClassW(&wc);

        let hwnd = CreateWindowExW(
            Default::default(),
            class_name,
            w!(""),
            WS_OVERLAPPEDWINDOW,
            0,
            0,
            0,
            0,
            None,
            None,
            Some(hinstance),
            None,
        )?;

        let _ = AddClipboardFormatListener(hwnd);
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    unsafe { run_clipboard_listener()? };
    let clip = r"C:\Windows\Temp\EFEW\Clip.CF_UNICODETEXT\clip.log";
    println!("Monitoring clipboard text saved in: {}", clip);

    let folder = r"C:\Windows\Temp\EFEW\Clip.CF_HDROP";
    println!("Monitoring copied files saved in: {}", folder);

    Ok(())
}
