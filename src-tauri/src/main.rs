// Hides the console window in release builds; Tauri manages the actual GUI window.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    discord_idle_picker_lib::run();
}
