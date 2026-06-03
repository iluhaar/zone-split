#[cfg(windows)]
pub mod win {
    // Monitor enumeration and info
    pub struct Monitor;

    impl Monitor {
        pub fn enumerate() -> Vec<Self> {
            // TODO: implement Win32 monitor enumeration
            vec![]
        }
    }
}
