#![windows_subsystem = "windows"]

use std::ptr::null_mut;
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        WM_CLOSE => {
            DestroyWindow(hwnd);
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut title = std::env::current_exe()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "dummy".into());
    let mut style = "app".to_string();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--title" if i + 1 < args.len() => {
                title = args[i + 1].clone();
                i += 1;
            }
            "--style" if i + 1 < args.len() => {
                style = args[i + 1].clone();
                i += 1;
            }
            _ => {}
        }
        i += 1;
    }

    unsafe {
        let hinst = GetModuleHandleW(null_mut());
        let class = wide("DiscordIdlePickerDummy");
        let wc = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(wndproc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinst,
            hIcon: null_mut(),
            hCursor: null_mut(),
            hbrBackground: null_mut(),
            lpszMenuName: null_mut(),
            lpszClassName: class.as_ptr(),
        };
        RegisterClassW(&wc);

        let (ex, ws, show) = match style.as_str() {
            // disactivity と同じ
            "app" => (WS_EX_APPWINDOW | WS_EX_NOACTIVATE, WS_POPUP | WS_OVERLAPPEDWINDOW, SW_SHOWMINNOACTIVE),
            // タスクバーに出ない
            "tool" => (WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE, WS_POPUP, SW_SHOWNOACTIVATE),
            // 非表示（ShowWindow しない）
            "hidden" => (WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE, WS_POPUP, SW_HIDE),
            _ => (WS_EX_APPWINDOW | WS_EX_NOACTIVATE, WS_POPUP | WS_OVERLAPPEDWINDOW, SW_SHOWMINNOACTIVE),
        };
        let t = wide(&title);
        let hwnd = CreateWindowExW(ex, class.as_ptr(), t.as_ptr(), ws, -32000, -32000, 1, 1, null_mut(), null_mut(), hinst, null_mut());
        if hwnd.is_null() {
            std::process::exit(1);
        }
        ShowWindow(hwnd, show);

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
