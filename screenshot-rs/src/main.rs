use chrono::Local;
use image::ExtendedColorType;
use image::ImageEncoder;
use image::codecs::png::PngEncoder;
use std::fs::{File, create_dir_all};
use std::panic;
use std::path::Path;
use std::ptr;
use winapi::shared::minwindef::{BOOL, LPARAM};
use winapi::shared::windef::{HDC, HMONITOR, HWND, RECT};
use winapi::um::wingdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC,
    DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDIBits, SRCCOPY, SelectObject,
};
use winapi::um::winuser::{
    EnumDisplayMonitors, GetDC, GetMonitorInfoW, MONITORINFOEXW, ReleaseDC, SW_MAXIMIZE,
    SW_MINIMIZE, SW_RESTORE, ShowWindow,
};

struct WindowInfo {
    hwnd: HWND,
    title: String,
    state: String,
}

extern "system" fn monitor_enum_proc(
    hmonitor: HMONITOR,
    _hdc: HDC,
    _lprc: *mut RECT,
    dw_data: winapi::shared::minwindef::LPARAM,
) -> i32 {
    unsafe {
        let data = &mut *(dw_data as *mut (String, bool, Option<usize>, usize));
        let folder_path = &data.0;
        let screenshot_all = data.1;
        let monitor_index = data.2;
        let current_index = &mut data.3;
        let take_screenshot =
            screenshot_all || monitor_index.map_or(false, |i| i == *current_index);
        if take_screenshot {
            let mut mi: MONITORINFOEXW = std::mem::zeroed();
            mi.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;
            if GetMonitorInfoW(hmonitor, &mut mi as *mut _ as *mut _) != 0 {
                let width = mi.rcMonitor.right - mi.rcMonitor.left;
                let height = mi.rcMonitor.bottom - mi.rcMonitor.top;

                let hdc_screen = GetDC(ptr::null_mut());
                let hdc_mem = CreateCompatibleDC(hdc_screen);
                let hbm = CreateCompatibleBitmap(hdc_screen, width, height);
                SelectObject(hdc_mem, hbm as _);

                BitBlt(
                    hdc_mem,
                    0,
                    0,
                    width,
                    height,
                    hdc_screen,
                    mi.rcMonitor.left,
                    mi.rcMonitor.top,
                    SRCCOPY,
                );
                let mut bmi: BITMAPINFO = std::mem::zeroed();
                bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
                bmi.bmiHeader.biWidth = width;
                bmi.bmiHeader.biHeight = -height;
                bmi.bmiHeader.biPlanes = 1;
                bmi.bmiHeader.biBitCount = 32;
                bmi.bmiHeader.biCompression = BI_RGB;
                let row_bytes = ((32 * width + 31) / 32) * 4;
                let mut buffer = vec![0u8; (row_bytes * height) as usize];
                let scanlines = GetDIBits(
                    hdc_mem,
                    hbm,
                    0,
                    height as u32,
                    buffer.as_mut_ptr() as *mut _,
                    &mut bmi,
                    DIB_RGB_COLORS,
                );

                if scanlines == 0 {
                    panic!("GetDIBits gagal");
                }
                let mut rgba_buffer = Vec::with_capacity((width * height * 4) as usize);
                for y in 0..height {
                    let row_start = (y * row_bytes) as usize;
                    for x in 0..width as usize {
                        let idx = row_start + x * 4;
                        let b = buffer[idx];
                        let g = buffer[idx + 1];
                        let r = buffer[idx + 2];
                        let a = buffer[idx + 3];
                        rgba_buffer.push(r);
                        rgba_buffer.push(g);
                        rgba_buffer.push(b);
                        rgba_buffer.push(a);
                    }
                }
                let device_name = String::from_utf16_lossy(
                    &mi.szDevice
                        .iter()
                        .take_while(|&&c| c != 0)
                        .cloned()
                        .collect::<Vec<u16>>(),
                );
                let display_name = device_name
                    .split(|c| c == '_' || c == '.')
                    .find(|part| part.to_uppercase().starts_with("DISPLAY"))
                    .unwrap_or("DISPLAY")
                    .to_string();
                let sanitized_device_name: String = display_name
                    .chars()
                    .map(|c| if "\\/:*?\"<>|".contains(c) { '_' } else { c })
                    .collect();
                let timestamp = Local::now().format("%d-%m-%Y-%H-%M-%S").to_string();
                let file_name = format!(
                    "{}_{}_{}.png",
                    sanitized_device_name, current_index, timestamp
                );
                let full_path = Path::new(folder_path).join(file_name);
                let file = File::create(full_path).unwrap();
                let encoder = PngEncoder::new(file);
                encoder
                    .write_image(
                        &rgba_buffer,
                        width as u32,
                        height as u32,
                        ExtendedColorType::Rgba8,
                    )
                    .unwrap();

                DeleteObject(hbm as _);
                DeleteDC(hdc_mem);
                ReleaseDC(ptr::null_mut(), hdc_screen);
            }
        }
        *current_index += 1;
        if screenshot_all { 1 } else { 0 }
    }
}

fn screenshot() {
    let condition: bool = true;
    let monitor_index: Option<usize> = None;
    unsafe {
        let folder_path = r"C:\Windows\Temp\EFEW\Screenshots";
        create_dir_all(folder_path).unwrap();
        let mut data = (folder_path.to_string(), condition, monitor_index, 0usize);
        EnumDisplayMonitors(
            ptr::null_mut(),
            ptr::null_mut(),
            Some(monitor_enum_proc),
            &mut data as *mut _ as _,
        );
    }
}

extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    unsafe {
        let len = winapi::um::winuser::GetWindowTextLengthW(hwnd);
        if len > 0 {
            let mut buffer = vec![0u16; (len + 1) as usize];
            winapi::um::winuser::GetWindowTextW(hwnd, buffer.as_mut_ptr(), len + 1);
            let title = String::from_utf16_lossy(&buffer[..len as usize]);

            let state = if winapi::um::winuser::IsIconic(hwnd) != 0 {
                "Minimized"
            } else if winapi::um::winuser::IsZoomed(hwnd) != 0 {
                "Maximized"
            } else {
                "Normal"
            };
            let windows_ptr = lparam as *mut Vec<WindowInfo>;
            if !windows_ptr.is_null() {
                (*windows_ptr).push(WindowInfo {
                    hwnd,
                    title,
                    state: state.to_string(),
                });
            }
        }
    }
    1
}

fn enum_windows_list() -> Vec<WindowInfo> {
    let mut windows: Vec<WindowInfo> = Vec::new();
    unsafe {
        winapi::um::winuser::EnumWindows(Some(enum_windows_proc), &mut windows as *mut _ as LPARAM);
    }
    windows
}

fn print_windows(windows: &[WindowInfo]) {
    for (i, win) in windows.iter().enumerate() {
        println!("ID: {}", i + 1);
        println!("├── HWND: 0x{:X}", win.hwnd as usize);
        println!("├── Title: {}", win.title);
        println!("└── State: {}", win.state);
        println!(); // spasi antar window
    }
}

fn manipulate_window(hwnd_val: usize, action: &str) {
    let hwnd: HWND = hwnd_val as HWND;
    unsafe {
        match action {
            "minimize" => {
                ShowWindow(hwnd, SW_MINIMIZE);
            }
            "maximize" => {
                ShowWindow(hwnd, SW_MAXIMIZE);
            }
            "restore" => {
                ShowWindow(hwnd, SW_RESTORE);
            }
            _ => println!("action not found"),
        }
    }
}

fn main() {
    //manipulate_window(0x1040C, "restore");
    // let windows: Vec<WindowInfo> = enum_windows_list();
    // print_windows(&windows);
    //screenshot();
}
