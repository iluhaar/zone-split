# zone-split — Windows Virtual Screen Splitter

A native Windows Rust app that splits your monitor into named zones and snaps windows into them.

## Architecture

```
zone-split/
├── src/
│   ├── main.rs          # Entry point
│   ├── overlay.rs       # Transparent drawing overlay (Win32, cfg-gated)
│   ├── zones.rs         # Pure geometry: Zone struct, hit-test, nearest-zone
│   ├── wm.rs            # Window toggle logic (save/restore positions)
│   ├── snap.rs          # Snap-to-zone logic
│   ├── persistence.rs   # Save/load zone layouts (JSON)
│   ├── hook.rs          # Win32 keyboard/mouse hooks (cfg-gated)
│   ├── tray.rs          # System tray icon/menu (Win32, cfg-gated)
│   ├── monitor.rs       # Monitor enumeration (Win32, cfg-gated)
│   └── ops.rs           # WindowOps trait + mock impls for testing
├── tests/               # Integration test stubs
├── assets/              # Icons and resources
└── .github/workflows/   # CI configuration
```

## Key Design Constraints

- **All Win32 / windows-rs code must be inside `#[cfg(windows)]` blocks** so that `cargo test --lib` passes on Linux.
- Pure logic modules (`zones.rs`, `wm.rs`, `snap.rs`, `persistence.rs`) have zero Win32 imports.
- `ops.rs` defines a `WindowOps` trait with `MockOps` / `MockOpsMut` for testing.

## Building

```sh
# Linux (tests only, no Win32 features)
cargo test --lib

# Windows (full build)
cargo build --release
cargo test
```

## Modules

### ops.rs
Defines portable `HWND` / `RECT` type aliases (real on Windows, plain structs on Linux) and the `WindowOps` trait. Provides `MockOps` (read-only) and `MockOpsMut` (tracks `set_pos` calls) for unit tests.

### zones.rs
Pure geometry module:
- `Zone { id: u32, rect: SerdeRect }` — serde-serializable zone
- `zone_from_drag(p1, p2) -> RECT` — normalize any drag direction
- `hit_test(point, zones) -> Option<u32>` — last zone wins on overlap
- `nearest_zone(point, zones) -> Option<u32>` — center-distance; lower index tiebreak
- `zones_overlap(a, b) -> bool` — strict area overlap

### wm.rs
`WindowManager` with a `HashMap<isize, RECT>` of saved positions.
- `toggle(hwnd, zone_rect, ops)` — expand or restore
- `on_destroy(hwnd)` — clean up saved state

### snap.rs
- `snap_target(center, zones)` — nearest zone center
- `snap_if_near_edge(center, zones, threshold)` — only snap when near an edge

### persistence.rs
- `save_layout(zones, path)` — write pretty JSON
- `load_layout(path)` — read JSON; missing file → `Ok(vec![])`, corrupt → `Err`

### hook.rs / overlay.rs / tray.rs / monitor.rs
Win32-only stubs, gated behind `#[cfg(windows)]`.

## CI

- `windows-latest`: fmt check, clippy, full test suite, release build
- `ubuntu-latest`: `cargo test --lib` (pure-logic tests only)
