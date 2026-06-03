use crate::zones::Zone;
use std::io;
use std::path::Path;

pub fn save_layout(zones: &[Zone], path: &Path) -> io::Result<()> {
    let json = serde_json::to_string_pretty(zones)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    std::fs::write(path, json)
}

pub fn load_layout(path: &Path) -> io::Result<Vec<Zone>> {
    match std::fs::read_to_string(path) {
        Ok(contents) => {
            let zones: Vec<Zone> = serde_json::from_str(&contents)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            Ok(zones)
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(vec![]),
        Err(e) => Err(e),
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
    fn test_save_and_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("layout.json");

        let zones = vec![
            make_zone(1, 0, 0, 960, 1080),
            make_zone(2, 960, 0, 1920, 1080),
        ];

        save_layout(&zones, &path).unwrap();
        let loaded = load_layout(&path).unwrap();
        assert_eq!(zones, loaded);
    }

    #[test]
    fn test_load_missing_file_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nonexistent.json");
        let result = load_layout(&path).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_load_corrupted_json_returns_err() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.json");
        std::fs::write(&path, b"not valid json {{{{").unwrap();
        let result = load_layout(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_save_empty_layout() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.json");
        save_layout(&[], &path).unwrap();
        let loaded = load_layout(&path).unwrap();
        assert!(loaded.is_empty());
    }
}
