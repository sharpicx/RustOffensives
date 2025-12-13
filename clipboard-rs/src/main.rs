use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Memory::GlobalLock;
use windows::Win32::System::Memory::GlobalUnlock;
use windows::{
    Win32::Foundation::*, Win32::System::DataExchange::*, Win32::UI::WindowsAndMessaging::*,
    core::*,
};

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
                        println!("[{}] {}", "+", text);
                        let _ = GlobalUnlock(hglobal);
                    }
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
    //    unsafe { run_clipboard_listener()? };

    Ok(())
}
