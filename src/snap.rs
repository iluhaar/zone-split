use crate::zones::{self, Zone};

/// Returns the zone id of the nearest zone center, or None if zones is empty.
pub fn snap_target(window_center: (i32, i32), zones: &[Zone]) -> Option<u32> {
    zones::nearest_zone(window_center, zones)
}

/// Only snap if window_center is within threshold pixels of any edge of the target zone.
pub fn snap_if_near_edge(window_center: (i32, i32), zones: &[Zone], threshold: i32) -> Option<u32> {
    let target_id = snap_target(window_center, zones)?;
    let zone = zones.iter().find(|z| z.id == target_id)?;
    let r = zone.rect;

    let near_left = (window_center.0 - r.left).abs() <= threshold;
    let near_right = (window_center.0 - r.right).abs() <= threshold;
    let near_top = (window_center.1 - r.top).abs() <= threshold;
    let near_bottom = (window_center.1 - r.bottom).abs() <= threshold;

    if near_left || near_right || near_top || near_bottom {
        Some(target_id)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zones::{SerdeRect, Zone};

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
    fn test_snap_target_basic() {
        let zones = vec![make_zone(1, 0, 0, 100, 100), make_zone(2, 200, 0, 300, 100)];
        assert_eq!(snap_target((40, 50), &zones), Some(1));
        assert_eq!(snap_target((260, 50), &zones), Some(2));
    }

    #[test]
    fn test_snap_target_equidistant_lower_index_wins() {
        let zones = vec![
            make_zone(1, 0, 0, 100, 100),   // center (50, 50)
            make_zone(2, 100, 0, 200, 100), // center (150, 50)
        ];
        // Point equidistant at x=100 — lower index (id=1) wins
        assert_eq!(snap_target((100, 50), &zones), Some(1));
    }

    #[test]
    fn test_snap_target_empty() {
        assert_eq!(snap_target((50, 50), &[]), None);
    }

    #[test]
    fn test_snap_if_near_edge_within_threshold() {
        let zones = vec![make_zone(1, 0, 0, 100, 100)];
        // Near left edge (x=0)
        assert_eq!(snap_if_near_edge((5, 50), &zones, 10), Some(1));
        // Near right edge (x=100)
        assert_eq!(snap_if_near_edge((95, 50), &zones, 10), Some(1));
        // Near top edge (y=0)
        assert_eq!(snap_if_near_edge((50, 5), &zones, 10), Some(1));
        // Near bottom edge (y=100)
        assert_eq!(snap_if_near_edge((50, 95), &zones, 10), Some(1));
    }

    #[test]
    fn test_snap_if_near_edge_center_no_snap() {
        let zones = vec![make_zone(1, 0, 0, 200, 200)];
        // Dead center, not near any edge with threshold=10
        assert_eq!(snap_if_near_edge((100, 100), &zones, 10), None);
    }

    #[test]
    fn test_snap_if_near_edge_empty() {
        assert_eq!(snap_if_near_edge((50, 50), &[], 10), None);
    }

    #[test]
    fn test_snap_if_near_edge_exact_threshold() {
        let zones = vec![make_zone(1, 0, 0, 100, 100)];
        // Exactly at threshold distance from left edge
        assert_eq!(snap_if_near_edge((10, 50), &zones, 10), Some(1));
        // One pixel beyond threshold
        assert_eq!(snap_if_near_edge((11, 50), &zones, 10), None);
    }
}
