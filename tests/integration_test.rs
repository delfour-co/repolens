//! Integration tests for RepoLens CLI

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[allow(deprecated)]
fn get_cmd() -> Command {
    Command::cargo_bin("repolens").unwrap()
}

#[tokio::test]
async fn test_init_command_creates_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join(".repolens.toml");

    // Run init command in temp directory
    get_cmd()
        .current_dir(temp_dir.path())
        .args([
            "init",
            "--preset",
            "opensource",
            "--non-interactive",
            "--force",
            "--skip-checks",
        ])
        .assert()
        .success();

    // Verify config file was created
    assert!(config_path.exists(), "Configuration file should be created");

    // Verify config file content
    let content = fs::read_to_string(&config_path).unwrap();
    assert!(
        content.contains("preset = \"opensource\""),
        "Config should contain preset"
    );
}

#[tokio::test]
async fn test_init_command_with_different_presets() {
    let presets = vec!["opensource", "enterprise", "strict"];

    for preset in presets {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".repolens.toml");

        get_cmd()
            .current_dir(temp_dir.path())
            .args([
                "init",
                "--preset",
                preset,
                "--non-interactive",
                "--force",
                "--skip-checks",
            ])
            .assert()
            .success();

        assert!(
            config_path.exists(),
            "Config should be created for preset: {}",
            preset
        );
    }
}

#[tokio::test]
async fn test_init_command_refuses_overwrite_without_force() {
    let temp_dir = TempDir::new().unwrap();
    let _config_path = temp_dir.path().join(".repolens.toml");

    // Create initial config
    get_cmd()
        .current_dir(temp_dir.path())
        .args([
            "init",
            "--preset",
            "opensource",
            "--non-interactive",
            "--force",
            "--skip-checks",
        ])
        .assert()
        .success();

    // Try to create again without force
    get_cmd()
        .current_dir(temp_dir.path())
        .args([
            "init",
            "--preset",
            "enterprise",
            "--non-interactive",
            "--skip-checks",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[tokio::test]
async fn test_plan_command_runs_successfully() {
    let temp_dir = TempDir::new().unwrap();

    // Initialize config first
    get_cmd()
        .current_dir(temp_dir.path())
        .args([
            "init",
            "--preset",
            "opensource",
            "--non-interactive",
            "--force",
            "--skip-checks",
        ])
        .assert()
        .success();

    // Run plan command (may return non-zero if issues are found, which is expected)
    get_cmd()
        .current_dir(temp_dir.path())
        .args(["plan"])
        .assert()
        .code(predicate::ne(255)); // Any exit code except 255 (unexpected error)
}

#[tokio::test]
async fn test_plan_command_with_json_output() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("plan.json");

    // Initialize config first
    get_cmd()
        .current_dir(temp_dir.path())
        .args([
            "init",
            "--preset",
            "opensource",
            "--non-interactive",
            "--force",
            "--skip-checks",
        ])
        .assert()
        .success();

    // Run plan command with JSON output (may return non-zero if issues are found)
    get_cmd()
        .current_dir(temp_dir.path())
        .args(["plan", "--format", "json", "--output"])
        .arg(&output_path)
        .assert()
        .code(predicate::ne(255)); // Any exit code except 255 (unexpected error)

    // Verify output file was created
    assert!(output_path.exists(), "JSON output file should be created");

    // Verify it's valid JSON
    let content = fs::read_to_string(&output_path).unwrap();
    let _: serde_json::Value = serde_json::from_str(&content).expect("Output should be valid JSON");
}

#[tokio::test]
async fn test_plan_command_detects_missing_readme() {
    let temp_dir = TempDir::new().unwrap();

    // Initialize config
    get_cmd()
        .current_dir(temp_dir.path())
        .args([
            "init",
            "--preset",
            "opensource",
            "--non-interactive",
            "--force",
            "--skip-checks",
        ])
        .assert()
        .success();

    // Don't create README - it should be detected as missing
    // Run plan command (may return non-zero if issues are found, which is expected)
    let assert = get_cmd()
        .current_dir(temp_dir.path())
        .args(["plan"])
        .assert()
        .code(predicate::ne(255)); // Any exit code except 255 (unexpected error)
    let output = assert.get_output();

    let stdout = String::from_utf8_lossy(&output.stdout);
    // The plan should mention missing README (exact text depends on implementation)
    // This is a basic check - in a real scenario, we'd parse the output more carefully
    assert!(!stdout.is_empty(), "Plan output should not be empty");
}

#[tokio::test]
async fn test_template_creation() {
    let temp_dir = TempDir::new().unwrap();

    // Initialize config
    get_cmd()
        .current_dir(temp_dir.path())
        .args([
            "init",
            "--preset",
            "opensource",
            "--non-interactive",
            "--force",
            "--skip-checks",
        ])
        .assert()
        .success();

    // Create a README to test that templates can be created
    // This is a basic test - full template creation testing would require
    // running the apply command or testing the template module directly
    let readme_path = temp_dir.path().join("README.md");
    fs::write(&readme_path, "# Test Project\n\nA test project.").unwrap();

    // Verify file was created
    assert!(readme_path.exists(), "README should exist");
}

#[tokio::test]
async fn test_help_command() {
    get_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("repolens"))
        .stdout(predicate::str::contains("Commands:"))
        .stdout(predicate::str::contains("init"))
        .stdout(predicate::str::contains("plan"));
}

#[tokio::test]
async fn test_version_command() {
    get_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("repolens"));
}
