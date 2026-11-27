//! imkitchen app: the application, on desktop, Android and iOS.
//!
//! One code base for the three: the interface is written in Slint (`ui/`)
//! and compiled into this library by the build script. The binary of the
//! desktops and of iOS calls [`run`]; Android loads the library from its
//! `NativeActivity` and calls `android_main`.

slint::include_modules!();

/// The window of the app, ready to be shown.
pub fn window() -> Result<AppWindow, slint::PlatformError> {
    let window = AppWindow::new()?;
    window
        .global::<About>()
        .set_version(env!("CARGO_PKG_VERSION").into());
    Ok(window)
}

/// Shows the app and returns once its window is closed.
pub fn run() -> Result<(), slint::PlatformError> {
    window()?.run()
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: slint::android::AndroidApp) {
    slint::android::init(app).unwrap();
    run().unwrap();
}
