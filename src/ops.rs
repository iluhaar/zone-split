// Portable HWND and RECT for test use
#[cfg(not(windows))]
pub type HWND = isize;
#[cfg(not(windows))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RECT {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[cfg(windows)]
pub use windows::Win32::Foundation::{HWND, RECT};

pub type Result<T> = std::result::Result<T, String>;

#[cfg(not(windows))]
pub fn hwnd_key(hwnd: HWND) -> isize {
    hwnd
}

#[cfg(windows)]
pub fn hwnd_key(hwnd: HWND) -> isize {
    hwnd.0 as isize
}

#[cfg(not(windows))]
pub fn hwnd_from_isize(value: isize) -> HWND {
    value
}

#[cfg(windows)]
pub fn hwnd_from_isize(value: isize) -> HWND {
    HWND(value as *mut std::ffi::c_void)
}

pub trait WindowOps {
    fn get_rect(&self, hwnd: HWND) -> Result<RECT>;
    fn set_pos(&self, hwnd: HWND, rect: RECT) -> Result<()>;
    fn get_foreground(&self) -> HWND;
    fn get_monitor_rect(&self, hwnd: HWND) -> Result<RECT>;
}

#[cfg(windows)]
pub struct Win32Ops;

#[cfg(windows)]
impl WindowOps for Win32Ops {
    fn get_rect(&self, hwnd: HWND) -> Result<RECT> {
        let mut rect = RECT::default();
        unsafe {
            windows::Win32::UI::WindowsAndMessaging::GetWindowRect(hwnd, &mut rect)
                .map_err(|err| format!("GetWindowRect failed: {err}"))?;
        }
        Ok(rect)
    }

    fn set_pos(&self, hwnd: HWND, rect: RECT) -> Result<()> {
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;

        unsafe {
            windows::Win32::UI::WindowsAndMessaging::SetWindowPos(
                hwnd,
                None,
                rect.left,
                rect.top,
                width,
                height,
                windows::Win32::UI::WindowsAndMessaging::SWP_NOACTIVATE
                    | windows::Win32::UI::WindowsAndMessaging::SWP_NOZORDER,
            )
            .map_err(|err| format!("SetWindowPos failed: {err}"))?;
        }

        Ok(())
    }

    fn get_foreground(&self) -> HWND {
        unsafe { windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow() }
    }

    fn get_monitor_rect(&self, hwnd: HWND) -> Result<RECT> {
        unsafe {
            let monitor = windows::Win32::Graphics::Gdi::MonitorFromWindow(
                hwnd,
                windows::Win32::Graphics::Gdi::MONITOR_DEFAULTTONEAREST,
            );

            let mut info = windows::Win32::Graphics::Gdi::MONITORINFO {
                cbSize: std::mem::size_of::<windows::Win32::Graphics::Gdi::MONITORINFO>() as u32,
                ..Default::default()
            };

            if !windows::Win32::Graphics::Gdi::GetMonitorInfoW(monitor, &mut info).as_bool() {
                return Err(format!(
                    "GetMonitorInfoW failed: {}",
                    windows::core::Error::from_win32()
                ));
            }

            Ok(info.rcWork)
        }
    }
}

use std::collections::HashMap;
pub struct MockOps {
    pub state: HashMap<isize, RECT>,
}

impl WindowOps for MockOps {
    fn get_rect(&self, hwnd: HWND) -> Result<RECT> {
        let key = hwnd_key(hwnd);
        self.state
            .get(&key)
            .copied()
            .ok_or_else(|| format!("hwnd {} not found", key))
    }
    fn set_pos(&self, hwnd: HWND, rect: RECT) -> Result<()> {
        // MockOps is read-only in tests; tests use MockOpsMut for mutation
        let _ = (hwnd, rect);
        Ok(())
    }
    fn get_foreground(&self) -> HWND {
        hwnd_from_isize(0)
    }
    fn get_monitor_rect(&self, _hwnd: HWND) -> Result<RECT> {
        Ok(RECT {
            left: 0,
            top: 0,
            right: 1920,
            bottom: 1080,
        })
    }
}

// Mutable version for tests that track set_pos calls
pub struct MockOpsMut {
    pub state: std::cell::RefCell<HashMap<isize, RECT>>,
    pub calls: std::cell::RefCell<Vec<(isize, RECT)>>,
}

impl MockOpsMut {
    pub fn new(initial: HashMap<isize, RECT>) -> Self {
        Self {
            state: std::cell::RefCell::new(initial),
            calls: std::cell::RefCell::new(vec![]),
        }
    }
}

impl WindowOps for MockOpsMut {
    fn get_rect(&self, hwnd: HWND) -> Result<RECT> {
        let key = hwnd_key(hwnd);
        self.state
            .borrow()
            .get(&key)
            .copied()
            .ok_or_else(|| format!("hwnd {} not found", key))
    }
    fn set_pos(&self, hwnd: HWND, rect: RECT) -> Result<()> {
        let key = hwnd_key(hwnd);
        self.state.borrow_mut().insert(key, rect);
        self.calls.borrow_mut().push((key, rect));
        Ok(())
    }
    fn get_foreground(&self) -> HWND {
        hwnd_from_isize(0)
    }
    fn get_monitor_rect(&self, _hwnd: HWND) -> Result<RECT> {
        Ok(RECT {
            left: 0,
            top: 0,
            right: 1920,
            bottom: 1080,
        })
    }
}
