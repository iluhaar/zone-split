use crate::ops::{self, WindowOps, HWND, RECT};
use std::collections::HashMap;

pub struct WindowManager {
    saved: HashMap<isize, RECT>,
}

impl WindowManager {
    pub fn new() -> Self {
        Self {
            saved: HashMap::new(),
        }
    }

    /// If hwnd not in saved map: save current rect, expand to zone_rect.
    /// If hwnd in saved map: restore saved rect, remove from map.
    pub fn toggle(
        &mut self,
        hwnd: HWND,
        zone_rect: RECT,
        ops: &impl WindowOps,
    ) -> ops::Result<()> {
        let key = hwnd as isize;
        if let Some(saved_rect) = self.saved.remove(&key) {
            // Restore
            ops.set_pos(hwnd, saved_rect)?;
        } else {
            // Save and expand
            let current = ops.get_rect(hwnd)?;
            self.saved.insert(key, current);
            ops.set_pos(hwnd, zone_rect)?;
        }
        Ok(())
    }

    pub fn on_destroy(&mut self, hwnd: HWND) {
        let key = hwnd as isize;
        self.saved.remove(&key);
    }
}

impl Default for WindowManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::{MockOpsMut, RECT};
    use std::collections::HashMap;

    fn rect(left: i32, top: i32, right: i32, bottom: i32) -> RECT {
        RECT {
            left,
            top,
            right,
            bottom,
        }
    }

    #[test]
    fn test_toggle_unknown_saves_and_expands() {
        let hwnd: HWND = 1;
        let original = rect(100, 100, 300, 300);
        let zone = rect(0, 0, 960, 1080);

        let mut state = HashMap::new();
        state.insert(1isize, original);
        let ops = MockOpsMut::new(state);
        let mut wm = WindowManager::new();

        wm.toggle(hwnd, zone, &ops).unwrap();

        // Should have saved original and set to zone
        let calls = ops.calls.borrow();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0], (1, zone));
        // saved map should have the original rect
        assert_eq!(wm.saved.get(&1), Some(&original));
    }

    #[test]
    fn test_toggle_fullscreen_restores() {
        let hwnd: HWND = 1;
        let original = rect(100, 100, 300, 300);
        let zone = rect(0, 0, 960, 1080);

        let mut state = HashMap::new();
        state.insert(1isize, original);
        let ops = MockOpsMut::new(state);
        let mut wm = WindowManager::new();

        // First toggle: expand
        wm.toggle(hwnd, zone, &ops).unwrap();
        // Second toggle: restore
        wm.toggle(hwnd, zone, &ops).unwrap();

        let calls = ops.calls.borrow();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[1], (1, original));
        // saved map should be empty after restore
        assert!(!wm.saved.contains_key(&1));
    }

    #[test]
    fn test_toggle_twice_returns_to_original() {
        let hwnd: HWND = 42;
        let original = rect(50, 50, 200, 200);
        let zone = rect(0, 0, 1920, 1080);

        let mut state = HashMap::new();
        state.insert(42isize, original);
        let ops = MockOpsMut::new(state);
        let mut wm = WindowManager::new();

        wm.toggle(hwnd, zone, &ops).unwrap();
        wm.toggle(hwnd, zone, &ops).unwrap();

        let calls = ops.calls.borrow();
        assert_eq!(calls[0].1, zone);
        assert_eq!(calls[1].1, original);
    }

    #[test]
    fn test_on_destroy_removes_entry() {
        let hwnd: HWND = 5;
        let original = rect(0, 0, 100, 100);
        let zone = rect(0, 0, 960, 1080);

        let mut state = HashMap::new();
        state.insert(5isize, original);
        let ops = MockOpsMut::new(state);
        let mut wm = WindowManager::new();

        wm.toggle(hwnd, zone, &ops).unwrap();
        assert!(wm.saved.contains_key(&5));

        wm.on_destroy(hwnd);
        assert!(!wm.saved.contains_key(&5));
    }
}
