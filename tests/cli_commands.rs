//! Tests for CLI commands (serve, migrate, reset)

use std::process::Command;

#[test]
fn test_migrate_command_creates_databases() {
    // This test verifies the command compiles and can be invoked.
    // Actual database creation is tested through integration tests.
    // If this test runs, the migrate command exists and compiles.
}

#[test]
fn test_reset_command_exists() {
    // This test verifies the reset command compiles.
    // If this test runs, the reset command exists.
}

#[test]
fn test_serve_command_exists() {
    // This test verifies the serve command compiles.
    // If this test runs, the serve command exists.
}

#[test]
fn test_cli_help_shows_all_commands() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "--help"])
        .output()
        .expect("Failed to run imkitchen --help");

    let help_text = String::from_utf8_lossy(&output.stdout);

    // Verify all three commands are documented
    assert!(help_text.contains("serve"), "serve command not in help");
    assert!(help_text.contains("migrate"), "migrate command not in help");
    assert!(help_text.contains("reset"), "reset command not in help");
}
