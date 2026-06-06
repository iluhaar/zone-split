#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod hook;
mod monitor;
mod ops;
mod overlay;
mod persistence;
mod snap;
mod tray;
mod wm;
mod zones;

fn main() {
    run();
}

#[cfg(not(windows))]
fn run() {
    println!("zone-split is only functional on Windows");
}

#[cfg(windows)]
fn run() {
    if let Err(err) = windows_main() {
        log_event(&format!("zone-split failed: {err}"));
        eprintln!("zone-split failed: {err}");
    }
}

#[cfg(windows)]
fn windows_main() -> ops::Result<()> {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        MOD_ALT, MOD_CONTROL, MOD_SHIFT, MOD_WIN, VK_LEFT, VK_RIGHT, VK_UP,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, GetMessageW, TranslateMessage, MSG, WM_HOTKEY,
    };

    const HOTKEY_LEFT: i32 = 1;
    const HOTKEY_RIGHT: i32 = 2;
    const HOTKEY_FULL: i32 = 3;

    log_event("zone-split starting");

    let preferred = MOD_ALT | MOD_WIN;
    let fallback = MOD_CONTROL | MOD_ALT | MOD_SHIFT;
    register_hotkey(HOTKEY_LEFT, VK_LEFT.0 as u32, preferred, fallback, "left")?;
    register_hotkey(
        HOTKEY_RIGHT,
        VK_RIGHT.0 as u32,
        preferred,
        fallback,
        "right",
    )?;
    register_hotkey(HOTKEY_FULL, VK_UP.0 as u32, preferred, fallback, "full")?;

    let ops = ops::Win32Ops;
    let mut wm = wm::WindowManager::new();
    let mut msg = MSG::default();

    while unsafe { GetMessageW(&mut msg, HWND::default(), 0, 0).as_bool() } {
        if msg.message == WM_HOTKEY {
            let hotkey_id = msg.wParam.0 as i32;
            if let Err(err) = handle_hotkey(hotkey_id, &ops, &mut wm) {
                eprintln!("hotkey failed: {err}");
                log_event(&format!("hotkey failed: {err}"));
            }
        }

        unsafe {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    Ok(())
}

#[cfg(windows)]
fn register_hotkey(
    id: i32,
    key: u32,
    preferred: windows::Win32::UI::Input::KeyboardAndMouse::HOT_KEY_MODIFIERS,
    fallback: windows::Win32::UI::Input::KeyboardAndMouse::HOT_KEY_MODIFIERS,
    name: &str,
) -> ops::Result<()> {
    use windows::Win32::UI::Input::KeyboardAndMouse::RegisterHotKey;

    unsafe {
        match RegisterHotKey(None, id, preferred, key) {
            Ok(()) => {
                log_event(&format!("registered {name} hotkey: Win+Alt"));
                Ok(())
            }
            Err(preferred_err) => {
                log_event(&format!(
                    "Win+Alt {name} hotkey unavailable: {preferred_err}; trying Ctrl+Alt+Shift"
                ));
                RegisterHotKey(None, id, fallback, key).map_err(|fallback_err| {
                    format!(
                        "RegisterHotKey {name} failed; Win+Alt: {preferred_err}; Ctrl+Alt+Shift: {fallback_err}"
                    )
                })?;
                log_event(&format!("registered {name} hotkey: Ctrl+Alt+Shift"));
                Ok(())
            }
        }
    }
}

#[cfg(windows)]
fn log_event(message: &str) {
    use std::io::Write;

    let Ok(exe_path) = std::env::current_exe() else {
        return;
    };
    let Some(exe_dir) = exe_path.parent() else {
        return;
    };
    let log_path = exe_dir.join("zone-split.log");
    let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
    else {
        return;
    };

    let _ = writeln!(file, "{message}");
}

#[cfg(windows)]
fn handle_hotkey(
    hotkey_id: i32,
    ops: &ops::Win32Ops,
    wm: &mut wm::WindowManager,
) -> ops::Result<()> {
    use crate::ops::WindowOps;
    use windows::Win32::Foundation::RECT;

    let hwnd = ops.get_foreground();
    if ops::hwnd_key(hwnd) == 0 {
        return Err("no foreground window".to_string());
    }

    let monitor = ops.get_monitor_rect(hwnd)?;
    let mid_x = monitor.left + (monitor.right - monitor.left) / 2;
    let (zone_id, zone_rect) = match hotkey_id {
        1 => (
            1,
            RECT {
                left: monitor.left,
                top: monitor.top,
                right: mid_x,
                bottom: monitor.bottom,
            },
        ),
        2 => (
            2,
            RECT {
                left: mid_x,
                top: monitor.top,
                right: monitor.right,
                bottom: monitor.bottom,
            },
        ),
        3 => (3, monitor),
        _ => return Ok(()),
    };

    match wm.toggle(hwnd, zone_id, zone_rect, ops)? {
        wm::ToggleAction::Moved => set_window_border(hwnd, Some(0x0000B4FF)),
        wm::ToggleAction::Restored => set_window_border(hwnd, None),
    }
}

#[cfg(windows)]
fn set_window_border(
    hwnd: windows::Win32::Foundation::HWND,
    color: Option<u32>,
) -> ops::Result<()> {
    use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_BORDER_COLOR};

    const DWMWA_COLOR_DEFAULT: u32 = 0xFFFFFFFF;

    let value = color.unwrap_or(DWMWA_COLOR_DEFAULT);
    unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_BORDER_COLOR,
            &value as *const u32 as *const std::ffi::c_void,
            std::mem::size_of::<u32>() as u32,
        )
        .map_err(|err| format!("DwmSetWindowAttribute border failed: {err}"))?;
    }

    Ok(())
}
