#[cfg(windows)]
pub mod win {
    // Transparent overlay window for zone drawing
    pub struct Overlay;

    impl Overlay {
        pub fn new() -> Option<Self> {
            // TODO: implement Win32 overlay window
            None
        }
    }
}
