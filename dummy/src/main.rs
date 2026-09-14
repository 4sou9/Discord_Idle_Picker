// Fake game process. Discord detects a running game only when the process owns a
// *visible* window, so this creates a 1x1 tool window far off-screen (no taskbar
// entry, never activated) and pumps messages until it receives WM_CLOSE.
#![windows_subsystem = "windows"]

use std::ptr::null_mut;

use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW,
    PostQuitMessage, RegisterClassW, ShowWindow, TranslateMessage, MSG, SW_SHOWNOACTIVATE,
    WM_CLOSE, WM_DESTROY, WNDCLASSW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_POPUP,
};

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
    // Window title does not affect detection; use the exe name so it is recognizable.
    let title = std::env::current_exe()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "dummy".into());

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

        let title = wide(&title);
        let hwnd = CreateWindowExW(
            WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            class.as_ptr(),
            title.as_ptr(),
            WS_POPUP,
            -32000,
            -32000,
            1,
            1,
            null_mut(),
            null_mut(),
            hinst,
            null_mut(),
        );
        if hwnd.is_null() {
            std::process::exit(1);
        }
        // Must be visible: hidden windows are ignored by Discord's detection.
        ShowWindow(hwnd, SW_SHOWNOACTIVATE);

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
