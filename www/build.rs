//! Compiles the Tailwind stylesheet into `$OUT_DIR/tailwind.css`, served as
//! a topcoat asset through `topcoat::tailwind::stylesheet!()`.
//!
//! The step emits no `cargo:rerun-if-*` directive of its own, and any single
//! directive replaces Cargo's default "rerun on any package file". The
//! `rerun-if-changed` lines keep the stylesheet in sync when a utility class
//! is added to a `.rs` file; without them the class silently does not exist
//! at runtime.
fn main() {
    println!("cargo:rerun-if-changed=styles.css");
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-env-changed=TAILWIND_CLI");
    let mut tailwind = topcoat::tailwind::BuildConfig::new().input("styles.css");
    // The Nix dev shell exports the packaged CLI. Unset (CI, a plain cargo
    // build), topcoat downloads its pinned release into the target directory;
    // naming the variable while it is unset would be an error.
    if std::env::var_os("TAILWIND_CLI").is_some() {
        tailwind = tailwind.executable_env("TAILWIND_CLI");
    }
    tailwind.render().unwrap();
}
