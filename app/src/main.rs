//! The app on a desktop and on iOS. Android has no binary: see the library.

// No console behind the window on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() -> Result<(), slint::PlatformError> {
    imkitchen_app::run()
}
