//! Compiles the Slint sources of `ui/` into the Rust the library includes.
//!
//! The pictures and the translations are compiled into the binary: an app
//! installed on a phone has no directory of its own to read them from.
fn main() {
    let config = slint_build::CompilerConfiguration::new()
        // One look on every platform, instead of the one Slint would pick
        // for each.
        .with_style("material".into())
        .embed_resources(slint_build::EmbedResourcesKind::EmbedFiles)
        .with_bundled_translations("lang")
        // What the tests find the elements of a window with, left out of a
        // release.
        .with_debug_info(std::env::var("PROFILE").is_ok_and(|profile| profile == "debug"));
    slint_build::compile_with_config("ui/app.slint", config).unwrap();
}
