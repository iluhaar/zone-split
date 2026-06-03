#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod hook;
mod monitor;
mod ops;
mod overlay;
mod persistence;
mod snap;
mod tray;
mod wm;
mod zones;

fn main() {
    println!("zone-split starting");
}
