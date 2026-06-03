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

pub trait WindowOps {
    fn get_rect(&self, hwnd: HWND) -> Result<RECT>;
    fn set_pos(&self, hwnd: HWND, rect: RECT) -> Result<()>;
    fn get_foreground(&self) -> HWND;
    fn get_monitor_rect(&self, hwnd: HWND) -> Result<RECT>;
}

use std::collections::HashMap;
pub struct MockOps {
    pub state: HashMap<isize, RECT>,
}

impl WindowOps for MockOps {
    fn get_rect(&self, hwnd: HWND) -> Result<RECT> {
        let key = hwnd as isize;
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
        0 as HWND
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
        let key = hwnd as isize;
        self.state
            .borrow()
            .get(&key)
            .copied()
            .ok_or_else(|| format!("hwnd {} not found", key))
    }
    fn set_pos(&self, hwnd: HWND, rect: RECT) -> Result<()> {
        let key = hwnd as isize;
        self.state.borrow_mut().insert(key, rect);
        self.calls.borrow_mut().push((key, rect));
        Ok(())
    }
    fn get_foreground(&self) -> HWND {
        0 as HWND
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
