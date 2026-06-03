use crate::ops::RECT;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Zone {
    pub id: u32,
    pub rect: SerdeRect,
}

/// A serde-friendly rect (since ops::RECT on Windows is a windows-rs type)
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SerdeRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl From<SerdeRect> for RECT {
    fn from(r: SerdeRect) -> RECT {
        RECT {
            left: r.left,
            top: r.top,
            right: r.right,
            bottom: r.bottom,
        }
    }
}

impl From<RECT> for SerdeRect {
    fn from(r: RECT) -> SerdeRect {
        SerdeRect {
            left: r.left,
            top: r.top,
            right: r.right,
            bottom: r.bottom,
        }
    }
}

/// Normalize a drag from any direction into a RECT
pub fn zone_from_drag(p1: (i32, i32), p2: (i32, i32)) -> RECT {
    RECT {
        left: p1.0.min(p2.0),
        top: p1.1.min(p2.1),
        right: p1.0.max(p2.0),
        bottom: p1.1.max(p2.1),
    }
}

/// Returns zone id of the zone containing the point; edge pixels count as inside.
/// If multiple zones contain the point, the last one (highest index) wins.
pub fn hit_test(point: (i32, i32), zones: &[Zone]) -> Option<u32> {
    let mut result = None;
    for z in zones {
        let r = z.rect;
        if point.0 >= r.left && point.0 <= r.right && point.1 >= r.top && point.1 <= r.bottom {
            result = Some(z.id);
        }
    }
    result
}

/// Returns the zone id with the nearest center. Tiebreak: lower index wins.
pub fn nearest_zone(point: (i32, i32), zones: &[Zone]) -> Option<u32> {
    zones
        .iter()
        .enumerate()
        .map(|(i, z)| {
            let cx = (z.rect.left + z.rect.right) / 2;
            let cy = (z.rect.top + z.rect.bottom) / 2;
            let dx = (point.0 - cx) as i64;
            let dy = (point.1 - cy) as i64;
            let dist2 = dx * dx + dy * dy;
            (dist2, i, z.id)
        })
        .min_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)))
        .map(|(_, _, id)| id)
}

/// Returns true if the two RECTs have any overlapping area
pub fn zones_overlap(a: &RECT, b: &RECT) -> bool {
    a.left < b.right && a.right > b.left && a.top < b.bottom && a.bottom > b.top
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_zone(id: u32, left: i32, top: i32, right: i32, bottom: i32) -> Zone {
        Zone {
            id,
            rect: SerdeRect {
                left,
                top,
                right,
                bottom,
            },
        }
    }

    #[test]
    fn test_zone_from_drag_normal() {
        let r = zone_from_drag((10, 20), (100, 200));
        assert_eq!(r.left, 10);
        assert_eq!(r.top, 20);
        assert_eq!(r.right, 100);
        assert_eq!(r.bottom, 200);
    }

    #[test]
    fn test_zone_from_drag_reversed() {
        let r = zone_from_drag((100, 200), (10, 20));
        assert_eq!(r.left, 10);
        assert_eq!(r.top, 20);
        assert_eq!(r.right, 100);
        assert_eq!(r.bottom, 200);
    }

    #[test]
    fn test_zone_from_drag_diagonal() {
        let r = zone_from_drag((50, 300), (200, 100));
        assert_eq!(r.left, 50);
        assert_eq!(r.top, 100);
        assert_eq!(r.right, 200);
        assert_eq!(r.bottom, 300);
    }

    #[test]
    fn test_hit_test_inside() {
        let zones = vec![make_zone(1, 0, 0, 100, 100)];
        assert_eq!(hit_test((50, 50), &zones), Some(1));
    }

    #[test]
    fn test_hit_test_edge() {
        let zones = vec![make_zone(1, 0, 0, 100, 100)];
        assert_eq!(hit_test((0, 0), &zones), Some(1));
        assert_eq!(hit_test((100, 100), &zones), Some(1));
        assert_eq!(hit_test((0, 100), &zones), Some(1));
        assert_eq!(hit_test((100, 0), &zones), Some(1));
    }

    #[test]
    fn test_hit_test_outside() {
        let zones = vec![make_zone(1, 0, 0, 100, 100)];
        assert_eq!(hit_test((101, 50), &zones), None);
        assert_eq!(hit_test((50, 101), &zones), None);
    }

    #[test]
    fn test_hit_test_overlap_last_wins() {
        let zones = vec![make_zone(1, 0, 0, 100, 100), make_zone(2, 50, 50, 150, 150)];
        // Point in overlap: last in vec (id=2) wins
        assert_eq!(hit_test((75, 75), &zones), Some(2));
        // Point only in first zone
        assert_eq!(hit_test((10, 10), &zones), Some(1));
    }

    #[test]
    fn test_hit_test_empty() {
        assert_eq!(hit_test((50, 50), &[]), None);
    }

    #[test]
    fn test_nearest_zone_basic() {
        let zones = vec![
            make_zone(1, 0, 0, 100, 100),   // center (50, 50)
            make_zone(2, 200, 0, 300, 100), // center (250, 50)
        ];
        assert_eq!(nearest_zone((60, 50), &zones), Some(1));
        assert_eq!(nearest_zone((240, 50), &zones), Some(2));
    }

    #[test]
    fn test_nearest_zone_tiebreak_lower_index() {
        let zones = vec![
            make_zone(1, 0, 0, 100, 100),   // center (50, 50)
            make_zone(2, 100, 0, 200, 100), // center (150, 50)
        ];
        // Equidistant at x=100
        assert_eq!(nearest_zone((100, 50), &zones), Some(1));
    }

    #[test]
    fn test_nearest_zone_empty() {
        assert_eq!(nearest_zone((50, 50), &[]), None);
    }

    #[test]
    fn test_zones_overlap_yes() {
        let a = RECT {
            left: 0,
            top: 0,
            right: 100,
            bottom: 100,
        };
        let b = RECT {
            left: 50,
            top: 50,
            right: 150,
            bottom: 150,
        };
        assert!(zones_overlap(&a, &b));
    }

    #[test]
    fn test_zones_overlap_no() {
        let a = RECT {
            left: 0,
            top: 0,
            right: 100,
            bottom: 100,
        };
        let b = RECT {
            left: 101,
            top: 0,
            right: 200,
            bottom: 100,
        };
        assert!(!zones_overlap(&a, &b));
    }

    #[test]
    fn test_zones_overlap_touching_edge_no_overlap() {
        let a = RECT {
            left: 0,
            top: 0,
            right: 100,
            bottom: 100,
        };
        let b = RECT {
            left: 100,
            top: 0,
            right: 200,
            bottom: 100,
        };
        // Touching but not overlapping (strict inequality check)
        assert!(!zones_overlap(&a, &b));
    }

    #[test]
    fn test_zone_serde_roundtrip() {
        let zone = make_zone(42, 10, 20, 300, 400);
        let json = serde_json::to_string(&zone).unwrap();
        let restored: Zone = serde_json::from_str(&json).unwrap();
        assert_eq!(zone, restored);
    }
}
