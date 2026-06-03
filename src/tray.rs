#[cfg(windows)]
pub mod win {
    // System tray icon and menu
    pub struct TrayIcon;

    impl TrayIcon {
        pub fn new() -> Option<Self> {
            // TODO: implement Win32 tray icon
            None
        }
    }
}
