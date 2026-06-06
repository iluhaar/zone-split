use crate::ops::{self, WindowOps, HWND, RECT};
use std::collections::HashMap;

pub struct WindowManager {
    saved: HashMap<isize, SavedWindow>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct SavedWindow {
    original_rect: RECT,
    zone_id: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ToggleAction {
    Moved,
    Restored,
}

impl WindowManager {
    pub fn new() -> Self {
        Self {
            saved: HashMap::new(),
        }
    }

    /// Same zone restores the original rect; a different zone moves directly there.
    pub fn toggle(
        &mut self,
        hwnd: HWND,
        zone_id: u32,
        zone_rect: RECT,
        ops: &impl WindowOps,
    ) -> ops::Result<ToggleAction> {
        let key = ops::hwnd_key(hwnd);
        if let Some(saved) = self.saved.get_mut(&key) {
            if saved.zone_id == zone_id {
                let original_rect = saved.original_rect;
                self.saved.remove(&key);
                ops.set_pos(hwnd, original_rect)?;
                Ok(ToggleAction::Restored)
            } else {
                saved.zone_id = zone_id;
                ops.set_pos(hwnd, zone_rect)?;
                Ok(ToggleAction::Moved)
            }
        } else {
            let current = ops.get_rect(hwnd)?;
            self.saved.insert(
                key,
                SavedWindow {
                    original_rect: current,
                    zone_id,
                },
            );
            ops.set_pos(hwnd, zone_rect)?;
            Ok(ToggleAction::Moved)
        }
    }

    pub fn on_destroy(&mut self, hwnd: HWND) {
        let key = ops::hwnd_key(hwnd);
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
    use crate::ops::{hwnd_from_isize, MockOpsMut, RECT};
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
        let hwnd = hwnd_from_isize(1);
        let original = rect(100, 100, 300, 300);
        let zone = rect(0, 0, 960, 1080);

        let mut state = HashMap::new();
        state.insert(1isize, original);
        let ops = MockOpsMut::new(state);
        let mut wm = WindowManager::new();

        assert_eq!(wm.toggle(hwnd, 1, zone, &ops).unwrap(), ToggleAction::Moved);

        // Should have saved original and set to zone
        let calls = ops.calls.borrow();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0], (1, zone));
        // saved map should have the original rect
        assert_eq!(
            wm.saved.get(&1).map(|saved| saved.original_rect),
            Some(original)
        );
    }

    #[test]
    fn test_toggle_fullscreen_restores() {
        let hwnd = hwnd_from_isize(1);
        let original = rect(100, 100, 300, 300);
        let zone = rect(0, 0, 960, 1080);

        let mut state = HashMap::new();
        state.insert(1isize, original);
        let ops = MockOpsMut::new(state);
        let mut wm = WindowManager::new();

        // First toggle: expand
        assert_eq!(wm.toggle(hwnd, 1, zone, &ops).unwrap(), ToggleAction::Moved);
        // Second toggle: restore
        assert_eq!(
            wm.toggle(hwnd, 1, zone, &ops).unwrap(),
            ToggleAction::Restored
        );

        let calls = ops.calls.borrow();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[1], (1, original));
        // saved map should be empty after restore
        assert!(!wm.saved.contains_key(&1));
    }

    #[test]
    fn test_toggle_twice_returns_to_original() {
        let hwnd = hwnd_from_isize(42);
        let original = rect(50, 50, 200, 200);
        let zone = rect(0, 0, 1920, 1080);

        let mut state = HashMap::new();
        state.insert(42isize, original);
        let ops = MockOpsMut::new(state);
        let mut wm = WindowManager::new();

        assert_eq!(wm.toggle(hwnd, 1, zone, &ops).unwrap(), ToggleAction::Moved);
        assert_eq!(
            wm.toggle(hwnd, 1, zone, &ops).unwrap(),
            ToggleAction::Restored
        );

        let calls = ops.calls.borrow();
        assert_eq!(calls[0].1, zone);
        assert_eq!(calls[1].1, original);
    }

    #[test]
    fn test_on_destroy_removes_entry() {
        let hwnd = hwnd_from_isize(5);
        let original = rect(0, 0, 100, 100);
        let zone = rect(0, 0, 960, 1080);

        let mut state = HashMap::new();
        state.insert(5isize, original);
        let ops = MockOpsMut::new(state);
        let mut wm = WindowManager::new();

        assert_eq!(wm.toggle(hwnd, 1, zone, &ops).unwrap(), ToggleAction::Moved);
        assert!(wm.saved.contains_key(&5));

        wm.on_destroy(hwnd);
        assert!(!wm.saved.contains_key(&5));
    }

    #[test]
    fn test_different_zone_moves_without_restoring() {
        let hwnd = hwnd_from_isize(7);
        let original = rect(50, 50, 200, 200);
        let left = rect(0, 0, 960, 1080);
        let right = rect(960, 0, 1920, 1080);

        let mut state = HashMap::new();
        state.insert(7isize, original);
        let ops = MockOpsMut::new(state);
        let mut wm = WindowManager::new();

        assert_eq!(wm.toggle(hwnd, 1, left, &ops).unwrap(), ToggleAction::Moved);
        assert_eq!(
            wm.toggle(hwnd, 2, right, &ops).unwrap(),
            ToggleAction::Moved
        );

        let calls = ops.calls.borrow();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0], (7, left));
        assert_eq!(calls[1], (7, right));
        assert_eq!(
            wm.saved.get(&7).map(|saved| saved.original_rect),
            Some(original)
        );
        assert_eq!(wm.saved.get(&7).map(|saved| saved.zone_id), Some(2));
    }
}
