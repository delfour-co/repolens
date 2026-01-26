//! Prerequisites checking for RepoLens initialization
//!
//! This module verifies that required tools and configurations are available
//! before running RepoLens commands.

use anyhow::{bail, Context, Result};
use colored::Colorize;
use std::path::Path;
use std::process::Command;

/// Level of importance for a prerequisite check
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckLevel {
    /// Required for operation - failure blocks execution
    Required,
    /// Optional - failure generates a warning
    Optional,
}

/// Status of a prerequisite check
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckStatus {
    /// Check passed successfully
    Ok,
    /// Check failed
    Failed,
    /// Check was skipped (due to dependency failure)
    Skipped,
}

/// Result of a single prerequisite check
#[derive(Debug, Clone)]
pub struct CheckResult {
    /// Name of the check
    pub name: String,
    /// Whether this check is required or optional
    pub level: CheckLevel,
    /// Status of the check
    pub status: CheckStatus,
    /// Human-readable message (shown on failure)
    pub message: Option<String>,
    /// Suggested fix for the issue
    pub fix: Option<String>,
}

impl CheckResult {
    /// Create a successful check result
    pub fn ok(name: &str, level: CheckLevel) -> Self {
        Self {
            name: name.to_string(),
            level,
            status: CheckStatus::Ok,
            message: None,
            fix: None,
        }
    }

    /// Create a failed check result
    pub fn failed(name: &str, level: CheckLevel, message: &str, fix: Option<&str>) -> Self {
        Self {
            name: name.to_string(),
            level,
            status: CheckStatus::Failed,
            message: Some(message.to_string()),
            fix: fix.map(|s| s.to_string()),
        }
    }

    /// Create a skipped check result
    pub fn skipped(name: &str, level: CheckLevel) -> Self {
        Self {
            name: name.to_string(),
            level,
            status: CheckStatus::Skipped,
            message: None,
            fix: None,
        }
    }

    /// Check if this result represents a failure
    #[allow(dead_code)]
    pub fn is_failed(&self) -> bool {
        self.status == CheckStatus::Failed
    }

    /// Check if this is a required check that failed
    pub fn is_required_failure(&self) -> bool {
        self.level == CheckLevel::Required && self.status == CheckStatus::Failed
    }

    /// Check if this is an optional check that failed
    pub fn is_optional_failure(&self) -> bool {
        self.level == CheckLevel::Optional && self.status == CheckStatus::Failed
    }
}

/// Aggregated report of all prerequisite checks
#[derive(Debug, Clone)]
pub struct PrerequisitesReport {
    /// All check results
    pub checks: Vec<CheckResult>,
}

impl PrerequisitesReport {
    /// Create a new empty report
    pub fn new() -> Self {
        Self { checks: Vec::new() }
    }

    /// Add a check result to the report
    pub fn add(&mut self, result: CheckResult) {
        self.checks.push(result);
    }

    /// Check if all required checks passed
    pub fn all_required_passed(&self) -> bool {
        !self.checks.iter().any(|c| c.is_required_failure())
    }

    /// Get all failed required checks
    pub fn required_failures(&self) -> Vec<&CheckResult> {
        self.checks
            .iter()
            .filter(|c| c.is_required_failure())
            .collect()
    }

    /// Get all failed optional checks (warnings)
    pub fn optional_failures(&self) -> Vec<&CheckResult> {
        self.checks
            .iter()
            .filter(|c| c.is_optional_failure())
            .collect()
    }

    /// Check if there are any warnings
    pub fn has_warnings(&self) -> bool {
        self.checks.iter().any(|c| c.is_optional_failure())
    }
}

impl Default for PrerequisitesReport {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Check functions
// ============================================================================

/// Check if git is installed
pub fn check_git_installed() -> CheckResult {
    let output = Command::new("git").arg("--version").output();

    match output {
        Ok(o) if o.status.success() => CheckResult::ok("Git installed", CheckLevel::Required),
        _ => CheckResult::failed(
            "Git installed",
            CheckLevel::Required,
            "Git is not installed",
            Some("Install git: https://git-scm.com/downloads"),
        ),
    }
}

/// Check if the current directory is a git repository
pub fn check_is_git_repo(root: &Path) -> CheckResult {
    let git_dir = root.join(".git");

    if git_dir.exists() {
        CheckResult::ok("Git repository", CheckLevel::Required)
    } else {
        CheckResult::failed(
            "Git repository",
            CheckLevel::Required,
            "Not a git repository",
            Some("Run: git init"),
        )
    }
}

/// Check if GitHub CLI (gh) is installed
pub fn check_gh_installed() -> CheckResult {
    let output = Command::new("gh").arg("--version").output();

    match output {
        Ok(o) if o.status.success() => {
            CheckResult::ok("GitHub CLI installed", CheckLevel::Required)
        }
        _ => CheckResult::failed(
            "GitHub CLI installed",
            CheckLevel::Required,
            "GitHub CLI (gh) is not installed",
            Some("Install gh: https://cli.github.com/"),
        ),
    }
}

/// Check if GitHub CLI is authenticated
pub fn check_gh_authenticated() -> CheckResult {
    let output = Command::new("gh").args(["auth", "status"]).output();

    match output {
        Ok(o) if o.status.success() => {
            CheckResult::ok("GitHub CLI authenticated", CheckLevel::Required)
        }
        _ => CheckResult::failed(
            "GitHub CLI authenticated",
            CheckLevel::Required,
            "GitHub CLI is not authenticated",
            Some("Run: gh auth login"),
        ),
    }
}

/// Check if a remote origin is configured
pub fn check_remote_origin(root: &Path) -> CheckResult {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(root)
        .output();

    match output {
        Ok(o) if o.status.success() => {
            CheckResult::ok("Remote origin configured", CheckLevel::Optional)
        }
        _ => CheckResult::failed(
            "Remote origin configured",
            CheckLevel::Optional,
            "No remote origin configured",
            Some("Run: git remote add origin <url>"),
        ),
    }
}

/// Check if the remote origin is a GitHub repository
pub fn check_remote_is_github(root: &Path) -> CheckResult {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(root)
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let url = String::from_utf8_lossy(&o.stdout);
            if url.contains("github.com") {
                CheckResult::ok("Remote is GitHub", CheckLevel::Optional)
            } else {
                CheckResult::failed(
                    "Remote is GitHub",
                    CheckLevel::Optional,
                    "Remote origin is not a GitHub repository",
                    Some("RepoLens works best with GitHub repositories"),
                )
            }
        }
        _ => CheckResult::skipped("Remote is GitHub", CheckLevel::Optional),
    }
}

// ============================================================================
// Run all checks
// ============================================================================

/// Options for running prerequisite checks
#[derive(Debug, Clone, Default)]
pub struct CheckOptions {
    /// Skip optional checks
    #[allow(dead_code)]
    pub skip_optional: bool,
}

/// Run all prerequisite checks
pub fn run_all_checks(root: &Path, _options: &CheckOptions) -> PrerequisitesReport {
    let mut report = PrerequisitesReport::new();

    // Required checks
    let git_installed = check_git_installed();
    let git_ok = git_installed.status == CheckStatus::Ok;
    report.add(git_installed);

    if git_ok {
        report.add(check_is_git_repo(root));
    } else {
        report.add(CheckResult::skipped("Git repository", CheckLevel::Required));
    }

    let gh_installed = check_gh_installed();
    let gh_ok = gh_installed.status == CheckStatus::Ok;
    report.add(gh_installed);

    if gh_ok {
        report.add(check_gh_authenticated());
    } else {
        report.add(CheckResult::skipped(
            "GitHub CLI authenticated",
            CheckLevel::Required,
        ));
    }

    // Optional checks (only if git repo exists)
    let is_repo = report
        .checks
        .iter()
        .find(|c| c.name == "Git repository")
        .map(|c| c.status == CheckStatus::Ok)
        .unwrap_or(false);

    if is_repo {
        let remote_result = check_remote_origin(root);
        let has_remote = remote_result.status == CheckStatus::Ok;
        report.add(remote_result);

        if has_remote {
            report.add(check_remote_is_github(root));
        } else {
            report.add(CheckResult::skipped(
                "Remote is GitHub",
                CheckLevel::Optional,
            ));
        }
    } else {
        report.add(CheckResult::skipped(
            "Remote origin configured",
            CheckLevel::Optional,
        ));
        report.add(CheckResult::skipped(
            "Remote is GitHub",
            CheckLevel::Optional,
        ));
    }

    report
}

// ============================================================================
// Display functions
// ============================================================================

/// Display the full prerequisites report
pub fn display_report(report: &PrerequisitesReport, _verbose: bool) {
    println!("{}\n", "Checking prerequisites...".bold());

    for check in &report.checks {
        let icon = match check.status {
            CheckStatus::Ok => "✓".green(),
            CheckStatus::Failed if check.level == CheckLevel::Required => "✗".red(),
            CheckStatus::Failed => "!".yellow(),
            CheckStatus::Skipped => "○".dimmed(),
        };

        let name = match check.status {
            CheckStatus::Ok => check.name.normal(),
            CheckStatus::Failed if check.level == CheckLevel::Required => check.name.red(),
            CheckStatus::Failed => check.name.yellow(),
            CheckStatus::Skipped => check.name.dimmed(),
        };

        let suffix = match check.status {
            CheckStatus::Skipped => " (skipped)".dimmed().to_string(),
            CheckStatus::Failed if check.level == CheckLevel::Optional => {
                " (optional)".dimmed().to_string()
            }
            _ => String::new(),
        };

        println!("  {} {}{}", icon, name, suffix);

        // Show message and fix for failures
        if check.status == CheckStatus::Failed {
            if let Some(msg) = &check.message {
                println!("    {}", msg.dimmed());
            }
            if let Some(fix) = &check.fix {
                println!("    {}: {}", "Fix".cyan(), fix);
            }
        }
    }

    println!();
}

/// Display error summary for failed required checks
pub fn display_error_summary(report: &PrerequisitesReport) {
    let failures = report.required_failures();
    if failures.is_empty() {
        return;
    }

    eprintln!(
        "{} {} required prerequisite(s) failed:",
        "Error:".red().bold(),
        failures.len()
    );

    for check in failures {
        eprintln!("  {} {}", "•".red(), check.name);
        if let Some(fix) = &check.fix {
            eprintln!("    {}: {}", "Fix".cyan(), fix);
        }
    }
}

/// Display warnings for failed optional checks
pub fn display_warnings(report: &PrerequisitesReport) {
    let warnings = report.optional_failures();
    if warnings.is_empty() {
        return;
    }

    println!(
        "{} {} optional check(s) failed:",
        "Warning:".yellow().bold(),
        warnings.len()
    );

    for check in warnings {
        if let Some(msg) = &check.message {
            println!("  {} {}", "•".yellow(), msg);
        }
    }

    println!();
}

// ============================================================================
// Centralized utility functions (used by other modules)
// ============================================================================

/// Check if gh CLI is available and authenticated
pub fn is_gh_available() -> bool {
    Command::new("gh")
        .args(["auth", "status"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Get repository info (owner/name) from GitHub CLI
pub fn get_repo_info() -> Result<String> {
    let output = Command::new("gh")
        .args([
            "repo",
            "view",
            "--json",
            "nameWithOwner",
            "-q",
            ".nameWithOwner",
        ])
        .output()
        .context("Failed to get repository info")?;

    if !output.status.success() {
        bail!("Failed to get repository info. Make sure you're in a git repository.");
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
