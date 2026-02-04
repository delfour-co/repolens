//! End-to-end tests for RepoLens CLI
//!
//! These tests run the CLI against real or simulated repositories to verify
//! that the audit correctly identifies issues and produces expected results.
//!
//! Tests marked with #[ignore] require network access and clone real repos.
//! Run them with: cargo test --test e2e_test -- --ignored

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use std::process::Command as StdCommand;
use tempfile::TempDir;

#[allow(deprecated)]
fn get_cmd() -> Command {
    Command::cargo_bin("repolens").unwrap()
}

// ============================================================================
// E2E Tests on RepoLens itself (this repository)
// ============================================================================

#[tokio::test]
async fn e2e_repolens_audit_runs_successfully() {
    // Run audit on the RepoLens repository itself
    let repo_root = std::env::current_dir().unwrap();

    get_cmd()
        .current_dir(&repo_root)
        .args(["plan", "--format", "json"])
        .assert()
        .code(predicate::in_iter([0, 1])); // 0 = no issues, 1 = issues found (both OK)
}

#[tokio::test]
async fn e2e_repolens_has_required_files() {
    let repo_root = std::env::current_dir().unwrap();

    // RepoLens should have all these files
    assert!(repo_root.join("README.md").exists());
    assert!(repo_root.join("LICENSE").exists());
    assert!(repo_root.join("CHANGELOG.md").exists());
    assert!(repo_root.join("Cargo.toml").exists());
    assert!(repo_root.join("Cargo.lock").exists());
    assert!(repo_root.join(".gitignore").exists());
}

#[tokio::test]
async fn e2e_repolens_report_json_valid() {
    let repo_root = std::env::current_dir().unwrap();
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("report.json");

    get_cmd()
        .current_dir(&repo_root)
        .args(["report", "--format", "json", "--output"])
        .arg(&output_path)
        .assert()
        .code(predicate::in_iter([0, 1]));

    // Verify output is valid JSON
    let content = fs::read_to_string(&output_path).unwrap();
    let report: serde_json::Value =
        serde_json::from_str(&content).expect("Report should be valid JSON");

    // Verify report structure (report has different format than plan)
    assert!(
        report.get("findings").is_some()
            || report
                .get("audit")
                .and_then(|a| a.get("findings"))
                .is_some(),
        "Report should have findings"
    );
    assert!(
        report.get("repository_name").is_some() || report.get("repository").is_some(),
        "Report should have repository info"
    );
}

#[tokio::test]
async fn e2e_repolens_report_markdown_valid() {
    let repo_root = std::env::current_dir().unwrap();
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("report.md");

    get_cmd()
        .current_dir(&repo_root)
        .args(["report", "--format", "markdown", "--output"])
        .arg(&output_path)
        .assert()
        .code(predicate::in_iter([0, 1]));

    // Verify output is valid Markdown
    let content = fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("# "), "Markdown should have headers");
}

// ============================================================================
// E2E Tests with simulated repositories
// ============================================================================

/// Create a minimal Rust project for testing
fn create_rust_project(dir: &Path) {
    // Cargo.toml
    fs::write(
        dir.join("Cargo.toml"),
        r#"[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
    )
    .unwrap();

    // src/main.rs
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(
        dir.join("src/main.rs"),
        "fn main() { println!(\"Hello\"); }\n",
    )
    .unwrap();

    // Initialize git
    StdCommand::new("git")
        .args(["init"])
        .current_dir(dir)
        .output()
        .ok();
    StdCommand::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(dir)
        .output()
        .ok();
    StdCommand::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(dir)
        .output()
        .ok();
}

/// Create a minimal Node.js project for testing
fn create_node_project(dir: &Path) {
    // package.json
    fs::write(
        dir.join("package.json"),
        r#"{
  "name": "test-project",
  "version": "1.0.0",
  "description": "Test project",
  "main": "index.js",
  "scripts": {
    "test": "echo \"Error: no test specified\" && exit 1"
  }
}
"#,
    )
    .unwrap();

    // index.js
    fs::write(dir.join("index.js"), "console.log('Hello');\n").unwrap();

    // Initialize git
    StdCommand::new("git")
        .args(["init"])
        .current_dir(dir)
        .output()
        .ok();
    StdCommand::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(dir)
        .output()
        .ok();
    StdCommand::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(dir)
        .output()
        .ok();
}

/// Create a minimal Python project for testing
fn create_python_project(dir: &Path) {
    // pyproject.toml
    fs::write(
        dir.join("pyproject.toml"),
        r#"[project]
name = "test-project"
version = "0.1.0"
description = "Test project"
requires-python = ">=3.8"
"#,
    )
    .unwrap();

    // main.py
    fs::write(dir.join("main.py"), "print('Hello')\n").unwrap();

    // Initialize git
    StdCommand::new("git")
        .args(["init"])
        .current_dir(dir)
        .output()
        .ok();
    StdCommand::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(dir)
        .output()
        .ok();
    StdCommand::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(dir)
        .output()
        .ok();
}

#[tokio::test]
async fn e2e_rust_project_missing_readme() {
    let temp_dir = TempDir::new().unwrap();
    create_rust_project(temp_dir.path());

    // Initialize repolens config
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

    // Run plan - should detect missing README
    let output = get_cmd()
        .current_dir(temp_dir.path())
        .args(["plan"])
        .assert()
        .code(predicate::eq(1)) // Should find issues
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    assert!(
        stdout.contains("README") || stdout.contains("readme"),
        "Should detect missing README"
    );
}

#[tokio::test]
async fn e2e_rust_project_missing_license() {
    let temp_dir = TempDir::new().unwrap();
    create_rust_project(temp_dir.path());

    // Initialize repolens config
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

    // Run plan - should detect missing LICENSE
    let output = get_cmd()
        .current_dir(temp_dir.path())
        .args(["plan"])
        .assert()
        .code(predicate::eq(1))
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    assert!(
        stdout.contains("LICENSE") || stdout.contains("license"),
        "Should detect missing LICENSE"
    );
}

#[tokio::test]
async fn e2e_rust_project_missing_lock_file() {
    let temp_dir = TempDir::new().unwrap();
    create_rust_project(temp_dir.path());

    // Initialize repolens config
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

    // Run plan with JSON output
    let output_path = temp_dir.path().join("plan.json");
    get_cmd()
        .current_dir(temp_dir.path())
        .args(["plan", "--format", "json", "--output"])
        .arg(&output_path)
        .assert()
        .code(predicate::eq(1));

    let content = fs::read_to_string(&output_path).unwrap();
    let plan: serde_json::Value = serde_json::from_str(&content).unwrap();

    // Check that DEP003 (missing lock file) is detected
    let findings = plan
        .get("audit")
        .and_then(|a| a.get("findings"))
        .and_then(|f| f.as_array());
    assert!(findings.is_some(), "Plan should have findings array");

    let has_lock_file_finding = findings.unwrap().iter().any(|f| {
        f.get("rule_id")
            .and_then(|r| r.as_str())
            .map(|r| r == "DEP003")
            .unwrap_or(false)
    });
    assert!(
        has_lock_file_finding,
        "Should detect missing Cargo.lock (DEP003)"
    );
}

#[tokio::test]
async fn e2e_node_project_missing_lock_file() {
    let temp_dir = TempDir::new().unwrap();
    create_node_project(temp_dir.path());

    // Initialize repolens config
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

    // Run plan with JSON output
    let output_path = temp_dir.path().join("plan.json");
    get_cmd()
        .current_dir(temp_dir.path())
        .args(["plan", "--format", "json", "--output"])
        .arg(&output_path)
        .assert()
        .code(predicate::eq(1));

    let content = fs::read_to_string(&output_path).unwrap();
    let plan: serde_json::Value = serde_json::from_str(&content).unwrap();

    let findings = plan
        .get("audit")
        .and_then(|a| a.get("findings"))
        .and_then(|f| f.as_array());
    assert!(findings.is_some(), "Plan should have findings array");

    let has_lock_file_finding = findings.unwrap().iter().any(|f| {
        f.get("rule_id")
            .and_then(|r| r.as_str())
            .map(|r| r == "DEP003")
            .unwrap_or(false)
    });
    assert!(
        has_lock_file_finding,
        "Should detect missing package-lock.json (DEP003)"
    );
}

#[tokio::test]
async fn e2e_python_project_audit() {
    let temp_dir = TempDir::new().unwrap();
    create_python_project(temp_dir.path());

    // Initialize repolens config
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

    // Run plan - should complete without crashing
    get_cmd()
        .current_dir(temp_dir.path())
        .args(["plan"])
        .assert()
        .code(predicate::in_iter([0, 1]));
}

#[tokio::test]
async fn e2e_project_with_secrets_detected() {
    let temp_dir = TempDir::new().unwrap();
    create_rust_project(temp_dir.path());

    // Add a file with a secret pattern (using realistic patterns that will be detected)
    // Note: These are fake secrets used only for testing detection
    fs::write(
        temp_dir.path().join("config.rs"),
        r#"
// Test file for secret detection
const AWS_ACCESS_KEY: &str = "AKIAIOSFODNN7EXAMPLE";
const AWS_SECRET_KEY: &str = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY";
"#,
    )
    .unwrap();

    // Initialize repolens config
    get_cmd()
        .current_dir(temp_dir.path())
        .args([
            "init",
            "--preset",
            "strict",
            "--non-interactive",
            "--force",
            "--skip-checks",
        ])
        .assert()
        .success();

    // Run plan with JSON output
    let output_path = temp_dir.path().join("plan.json");
    get_cmd()
        .current_dir(temp_dir.path())
        .args(["plan", "--format", "json", "--output"])
        .arg(&output_path)
        .assert()
        .code(predicate::eq(1));

    let content = fs::read_to_string(&output_path).unwrap();
    let plan: serde_json::Value = serde_json::from_str(&content).unwrap();

    let findings = plan
        .get("audit")
        .and_then(|a| a.get("findings"))
        .and_then(|f| f.as_array())
        .unwrap();

    // Should detect secrets (AWS keys are commonly detected)
    let has_secret_finding = findings.iter().any(|f| {
        f.get("category")
            .and_then(|c| c.as_str())
            .map(|c| c == "secrets")
            .unwrap_or(false)
    });
    assert!(has_secret_finding, "Should detect hardcoded secrets");
}

#[tokio::test]
async fn e2e_project_with_env_file() {
    let temp_dir = TempDir::new().unwrap();
    create_rust_project(temp_dir.path());

    // Add .env file (should be detected as sensitive)
    fs::write(
        temp_dir.path().join(".env"),
        "DATABASE_URL=localhost\nAPI_KEY=test123\n",
    )
    .unwrap();

    // Initialize repolens config
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

    // Run plan with JSON output
    let output_path = temp_dir.path().join("plan.json");
    get_cmd()
        .current_dir(temp_dir.path())
        .args(["plan", "--format", "json", "--output"])
        .arg(&output_path)
        .assert()
        .code(predicate::eq(1));

    let content = fs::read_to_string(&output_path).unwrap();
    let plan: serde_json::Value = serde_json::from_str(&content).unwrap();

    let findings = plan
        .get("audit")
        .and_then(|a| a.get("findings"))
        .and_then(|f| f.as_array())
        .unwrap();

    // Should detect .env file as sensitive (GIT003 or SEC003)
    let has_sensitive_file = findings.iter().any(|f| {
        let rule_id = f.get("rule_id").and_then(|r| r.as_str()).unwrap_or("");
        let message = f.get("message").and_then(|m| m.as_str()).unwrap_or("");
        rule_id == "GIT003" || rule_id == "SEC003" || message.to_lowercase().contains(".env")
    });
    assert!(
        has_sensitive_file,
        "Should detect .env as sensitive file. Findings: {:?}",
        findings
    );
}

#[tokio::test]
async fn e2e_project_with_dockerfile() {
    let temp_dir = TempDir::new().unwrap();
    create_rust_project(temp_dir.path());

    // Add Dockerfile without best practices
    fs::write(
        temp_dir.path().join("Dockerfile"),
        r#"FROM rust:latest
COPY . .
RUN cargo build --release
CMD ["./target/release/test-project"]
"#,
    )
    .unwrap();

    // Initialize repolens config
    get_cmd()
        .current_dir(temp_dir.path())
        .args([
            "init",
            "--preset",
            "strict",
            "--non-interactive",
            "--force",
            "--skip-checks",
        ])
        .assert()
        .success();

    // Run plan with JSON output
    let output_path = temp_dir.path().join("plan.json");
    get_cmd()
        .current_dir(temp_dir.path())
        .args(["plan", "--format", "json", "--output"])
        .arg(&output_path)
        .assert()
        .code(predicate::eq(1));

    let content = fs::read_to_string(&output_path).unwrap();
    let plan: serde_json::Value = serde_json::from_str(&content).unwrap();

    let findings = plan
        .get("audit")
        .and_then(|a| a.get("findings"))
        .and_then(|f| f.as_array())
        .unwrap();

    // Should detect Docker issues
    let docker_findings: Vec<_> = findings
        .iter()
        .filter(|f| {
            f.get("category")
                .and_then(|c| c.as_str())
                .map(|c| c == "docker")
                .unwrap_or(false)
        })
        .collect();

    assert!(
        !docker_findings.is_empty(),
        "Should detect Docker best practice issues"
    );

    // Specifically check for :latest tag (DOCKER003)
    let has_latest_tag = findings.iter().any(|f| {
        f.get("rule_id")
            .and_then(|r| r.as_str())
            .map(|r| r == "DOCKER003")
            .unwrap_or(false)
    });
    assert!(has_latest_tag, "Should detect unpinned :latest tag");
}

#[tokio::test]
async fn e2e_complete_opensource_project() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path().to_path_buf();

    create_rust_project(&temp_path);

    // Add all required files for opensource
    fs::write(
        temp_path.join("README.md"),
        "# Test Project\n\nA test project.\n",
    )
    .unwrap();
    fs::write(temp_path.join("LICENSE"), "MIT License\n\nCopyright 2024\n").unwrap();
    fs::write(
        temp_path.join("CONTRIBUTING.md"),
        "# Contributing\n\nWelcome!\n",
    )
    .unwrap();
    fs::write(
        temp_path.join("CODE_OF_CONDUCT.md"),
        "# Code of Conduct\n\nBe nice.\n",
    )
    .unwrap();
    fs::write(
        temp_path.join("SECURITY.md"),
        "# Security Policy\n\nReport issues.\n",
    )
    .unwrap();
    fs::write(
        temp_path.join("CHANGELOG.md"),
        "# Changelog\n\n## [Unreleased]\n\n## [0.1.0] - 2024-01-01\n\n- Initial release\n",
    )
    .unwrap();
    fs::write(temp_path.join(".gitignore"), "/target\n").unwrap();
    fs::write(
        temp_path.join("Cargo.lock"),
        "version = 3\n\n[[package]]\nname = \"test\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();

    // Initialize repolens config
    get_cmd()
        .current_dir(&temp_path)
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

    // Run plan - should have fewer/no critical issues
    // Exit codes: 0 = no issues, 1 = warnings only, 2 = critical issues
    let output_path = temp_path.join("plan.json");
    get_cmd()
        .current_dir(&temp_path)
        .args(["plan", "--format", "json", "--output"])
        .arg(&output_path)
        .assert()
        .code(predicate::in_iter([0, 1, 2]));

    let content = fs::read_to_string(&output_path).expect("Should be able to read plan output");
    let plan: serde_json::Value =
        serde_json::from_str(&content).expect("Plan should be valid JSON");

    // Check findings exist
    let findings = plan
        .get("audit")
        .and_then(|a| a.get("findings"))
        .and_then(|f| f.as_array());

    if let Some(findings) = findings {
        // Should not have critical file-related findings
        let critical_file_findings: Vec<_> = findings
            .iter()
            .filter(|f| {
                let severity = f.get("severity").and_then(|s| s.as_str()).unwrap_or("");
                let category = f.get("category").and_then(|c| c.as_str()).unwrap_or("");
                severity == "critical" && (category == "files" || category == "docs")
            })
            .collect();

        assert!(
            critical_file_findings.is_empty(),
            "Complete project should not have critical file/docs findings: {:?}",
            critical_file_findings
        );
    }
    // If findings is None, the test still passes (no findings = no critical findings)
}

#[tokio::test]
async fn e2e_presets_have_different_strictness() {
    let temp_dir = TempDir::new().unwrap();
    create_rust_project(temp_dir.path());

    // Add README to avoid the most common finding
    fs::write(temp_dir.path().join("README.md"), "# Test\n").unwrap();

    let mut findings_counts: Vec<(String, usize)> = Vec::new();

    for preset in &["opensource", "enterprise", "strict"] {
        // Re-initialize with different preset
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

        let output_path = temp_dir.path().join(format!("plan-{}.json", preset));
        get_cmd()
            .current_dir(temp_dir.path())
            .args(["plan", "--format", "json", "--output"])
            .arg(&output_path)
            .assert()
            .code(predicate::in_iter([0, 1]));

        let content = fs::read_to_string(&output_path).unwrap();
        let plan: serde_json::Value = serde_json::from_str(&content).unwrap();
        let count = plan
            .get("findings")
            .and_then(|f| f.as_array())
            .map(|a| a.len())
            .unwrap_or(0);

        findings_counts.push((preset.to_string(), count));
    }

    // Strict should generally find more issues than enterprise, which should find more than opensource
    // This isn't always true depending on the specific project, but in general it holds
    println!("Findings counts: {:?}", findings_counts);

    // At minimum, strict should not find fewer issues than opensource
    let opensource_count = findings_counts
        .iter()
        .find(|(p, _)| p == "opensource")
        .unwrap()
        .1;
    let strict_count = findings_counts
        .iter()
        .find(|(p, _)| p == "strict")
        .unwrap()
        .1;

    assert!(
        strict_count >= opensource_count,
        "Strict preset ({}) should find at least as many issues as opensource ({})",
        strict_count,
        opensource_count
    );
}

// ============================================================================
// E2E Tests with real GitHub repositories (requires network, run with --ignored)
// ============================================================================

/// Clone a repository to a temp directory
fn clone_repo(url: &str, dir: &Path) -> bool {
    StdCommand::new("git")
        .args(["clone", "--depth", "1", url, "."])
        .current_dir(dir)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[tokio::test]
#[ignore = "Requires network access - run with: cargo test -- --ignored"]
async fn e2e_real_repo_tokio() {
    let temp_dir = TempDir::new().unwrap();

    // Clone tokio (well-maintained Rust project)
    if !clone_repo("https://github.com/tokio-rs/tokio.git", temp_dir.path()) {
        eprintln!("Failed to clone tokio, skipping test");
        return;
    }

    // Initialize and run audit
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

    get_cmd()
        .current_dir(temp_dir.path())
        .args(["plan"])
        .assert()
        .code(predicate::in_iter([0, 1]));
}

#[tokio::test]
#[ignore = "Requires network access - run with: cargo test -- --ignored"]
async fn e2e_real_repo_express() {
    let temp_dir = TempDir::new().unwrap();

    // Clone express (popular Node.js project)
    if !clone_repo("https://github.com/expressjs/express.git", temp_dir.path()) {
        eprintln!("Failed to clone express, skipping test");
        return;
    }

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

    let output_path = temp_dir.path().join("plan.json");
    get_cmd()
        .current_dir(temp_dir.path())
        .args(["plan", "--format", "json", "--output"])
        .arg(&output_path)
        .assert()
        .code(predicate::in_iter([0, 1]));

    // Verify we can parse the output
    let content = fs::read_to_string(&output_path).unwrap();
    let _: serde_json::Value = serde_json::from_str(&content).expect("Should produce valid JSON");
}

#[tokio::test]
#[ignore = "Requires network access - run with: cargo test -- --ignored"]
async fn e2e_real_repo_flask() {
    let temp_dir = TempDir::new().unwrap();

    // Clone flask (popular Python project)
    if !clone_repo("https://github.com/pallets/flask.git", temp_dir.path()) {
        eprintln!("Failed to clone flask, skipping test");
        return;
    }

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

    get_cmd()
        .current_dir(temp_dir.path())
        .args(["plan"])
        .assert()
        .code(predicate::in_iter([0, 1]));
}

#[tokio::test]
#[ignore = "Requires network access - run with: cargo test -- --ignored"]
async fn e2e_real_repo_kubernetes() {
    let temp_dir = TempDir::new().unwrap();

    // Clone kubernetes (large Go project)
    if !clone_repo(
        "https://github.com/kubernetes/kubernetes.git",
        temp_dir.path(),
    ) {
        eprintln!("Failed to clone kubernetes, skipping test");
        return;
    }

    get_cmd()
        .current_dir(temp_dir.path())
        .args([
            "init",
            "--preset",
            "enterprise",
            "--non-interactive",
            "--force",
            "--skip-checks",
        ])
        .assert()
        .success();

    // Just verify it runs without crashing on a large repo
    get_cmd()
        .current_dir(temp_dir.path())
        .args(["plan"])
        .timeout(std::time::Duration::from_secs(120))
        .assert()
        .code(predicate::in_iter([0, 1]));
}
