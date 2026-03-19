//! Git history quality rules
//!
//! This module provides rules for analyzing Git history quality:
//! - Non-conventional commit messages (HIST001)
//! - Giant commits (HIST002)
//! - Unsigned commits (HIST003)
//! - Force push detected on protected branch (HIST004)

use crate::config::Config;
use crate::error::RepoLensError;
use crate::providers::github::GitHubProvider;
use crate::rules::engine::RuleCategory;
use crate::rules::results::{Finding, Severity};
use crate::scanner::Scanner;

use regex::Regex;
use std::process::Command;

/// Number of recent commits to analyze
const COMMITS_TO_ANALYZE: usize = 100;
/// Threshold for giant commits (number of files changed)
const GIANT_COMMIT_THRESHOLD: usize = 50;

/// Rules for checking Git history quality
pub struct HistoryRules;

#[async_trait::async_trait]
impl RuleCategory for HistoryRules {
    fn name(&self) -> &'static str {
        "history"
    }

    async fn run(&self, scanner: &Scanner, config: &Config) -> Result<Vec<Finding>, RepoLensError> {
        let mut findings = Vec::new();
        let root = scanner.root().to_path_buf();

        if config.is_rule_enabled("history/conventional-commits") {
            findings.extend(check_conventional_commits(&root));
        }

        if config.is_rule_enabled("history/giant-commits") {
            findings.extend(check_giant_commits(&root));
        }

        if config.is_rule_enabled("history/unsigned-commits") {
            findings.extend(check_unsigned_commits(&root));
        }

        if config.is_rule_enabled("history/force-push") {
            findings.extend(check_force_push());
        }

        Ok(findings)
    }
}

/// HIST001: Check for non-conventional commit messages
fn check_conventional_commits(root: &std::path::Path) -> Vec<Finding> {
    let mut findings = Vec::new();

    let output = Command::new("git")
        .args(["log", &format!("-{}", COMMITS_TO_ANALYZE), "--format=%s"])
        .current_dir(root)
        .output();

    let output = match output {
        Ok(out) if out.status.success() => out,
        _ => return findings,
    };

    let messages = String::from_utf8_lossy(&output.stdout);
    let conventional_re = Regex::new(
        r"^(feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert)(\(.+\))?!?:\s",
    )
    .expect("Invalid regex");

    let total = messages.lines().count();
    let non_conventional = messages
        .lines()
        .filter(|line| !line.is_empty() && !conventional_re.is_match(line))
        .count();

    if total > 0 && non_conventional > 0 {
        let percentage = (non_conventional as f64 / total as f64 * 100.0).round() as u32;
        findings.push(
            Finding::new(
                "HIST001",
                "history",
                Severity::Info,
                format!(
                    "{}/{} commits ({percentage}%) do not follow conventional commit format",
                    non_conventional, total
                ),
            )
            .with_description(
                "Conventional Commits provide a structured format for commit messages \
                 (e.g., feat:, fix:, docs:). This enables automatic changelog generation, \
                 semantic versioning, and better git history readability.",
            )
            .with_remediation(
                "Adopt the Conventional Commits specification (https://www.conventionalcommits.org). \
                 Consider using commitlint or similar tools to enforce the format.",
            ),
        );
    }

    findings
}

/// HIST002: Check for giant commits (> 50 files changed)
fn check_giant_commits(root: &std::path::Path) -> Vec<Finding> {
    let mut findings = Vec::new();

    // Get commit hashes and file counts using numstat
    let output = Command::new("git")
        .args([
            "log",
            &format!("-{}", COMMITS_TO_ANALYZE),
            "--format=%H",
            "--shortstat",
        ])
        .current_dir(root)
        .output();

    let output = match output {
        Ok(out) if out.status.success() => out,
        _ => return findings,
    };

    let text = String::from_utf8_lossy(&output.stdout);
    let files_changed_re = Regex::new(r"(\d+) files? changed").expect("Invalid regex");

    let mut giant_count = 0;

    for line in text.lines() {
        if let Some(caps) = files_changed_re.captures(line) {
            if let Ok(files) = caps[1].parse::<usize>() {
                if files > GIANT_COMMIT_THRESHOLD {
                    giant_count += 1;
                }
            }
        }
    }

    if giant_count > 0 {
        findings.push(
            Finding::new(
                "HIST002",
                "history",
                Severity::Warning,
                format!(
                    "{} commit(s) with more than {} files changed",
                    giant_count, GIANT_COMMIT_THRESHOLD
                ),
            )
            .with_description(
                "Large commits changing many files are harder to review, understand, \
                 and bisect. They often indicate that multiple logical changes were \
                 bundled into a single commit.",
            )
            .with_remediation(
                "Break large changes into smaller, focused commits. Each commit should \
                 represent a single logical change. Use interactive rebase to split commits.",
            ),
        );
    }

    findings
}

/// HIST003: Check for unsigned commits (no GPG/SSH signature)
fn check_unsigned_commits(root: &std::path::Path) -> Vec<Finding> {
    let mut findings = Vec::new();

    let output = Command::new("git")
        .args(["log", &format!("-{}", COMMITS_TO_ANALYZE), "--format=%G?"])
        .current_dir(root)
        .output();

    let output = match output {
        Ok(out) if out.status.success() => out,
        _ => return findings,
    };

    let signatures = String::from_utf8_lossy(&output.stdout);
    let total = signatures.lines().count();
    // G = good signature, U = good but untrusted, E = expired, X = expired key
    // N = no signature, B = bad signature
    let unsigned = signatures.lines().filter(|line| *line == "N").count();

    if total > 0 && unsigned > 0 {
        let percentage = (unsigned as f64 / total as f64 * 100.0).round() as u32;
        findings.push(
            Finding::new(
                "HIST003",
                "history",
                Severity::Info,
                format!(
                    "{}/{} commits ({percentage}%) are not signed",
                    unsigned, total
                ),
            )
            .with_description(
                "Signed commits provide cryptographic verification of authorship. \
                 This helps prevent commit spoofing and increases trust in the code history.",
            )
            .with_remediation(
                "Configure GPG or SSH commit signing: \
                 git config commit.gpgsign true. \
                 See https://docs.github.com/en/authentication/managing-commit-signature-verification",
            ),
        );
    }

    findings
}

/// HIST004: Check for force push on protected branches (via reflog)
fn check_force_push() -> Vec<Finding> {
    let mut findings = Vec::new();

    if !GitHubProvider::is_available() {
        return findings;
    }

    let provider = match GitHubProvider::new() {
        Ok(p) => p,
        Err(_) => return findings,
    };

    // Check GitHub audit log for force pushes via the events API
    let output = Command::new("gh")
        .args([
            "api",
            &format!("repos/{}/{}/events", provider.owner(), provider.name()),
            "--jq",
            r#"[.[] | select(.type == "PushEvent" and .payload.forced == true)] | length"#,
        ])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let count_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if let Ok(count) = count_str.parse::<u64>() {
                if count > 0 {
                    findings.push(
                        Finding::new(
                            "HIST004",
                            "history",
                            Severity::Warning,
                            format!(
                                "{} force push event(s) detected in recent activity",
                                count
                            ),
                        )
                        .with_description(
                            "Force pushes rewrite Git history and can cause data loss \
                             for other contributors. They should be avoided on shared branches, \
                             especially protected ones.",
                        )
                        .with_remediation(
                            "Enable branch protection rules to block force pushes on main branches. \
                             Use --force-with-lease instead of --force when necessary.",
                        ),
                    );
                }
            }
        }
        _ => {} // Skip if API call fails
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn init_git_repo(dir: &std::path::Path) {
        Command::new("git")
            .args(["init"])
            .current_dir(dir)
            .output()
            .expect("Failed to init git repo");
        Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(dir)
            .output()
            .expect("Failed to set git config");
        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(dir)
            .output()
            .expect("Failed to set git config");
    }

    #[test]
    fn test_conventional_commits_all_valid() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        init_git_repo(root);

        // Create conventional commits
        std::fs::write(root.join("file1.txt"), "content").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(root)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "feat: add feature"])
            .current_dir(root)
            .output()
            .unwrap();

        std::fs::write(root.join("file2.txt"), "content").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(root)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "fix: resolve bug"])
            .current_dir(root)
            .output()
            .unwrap();

        let findings = check_conventional_commits(root);
        assert!(
            findings.is_empty(),
            "Expected no findings for valid conventional commits"
        );
    }

    #[test]
    fn test_conventional_commits_some_invalid() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        init_git_repo(root);

        std::fs::write(root.join("file1.txt"), "content").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(root)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "feat: valid commit"])
            .current_dir(root)
            .output()
            .unwrap();

        std::fs::write(root.join("file2.txt"), "content").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(root)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "just a random message"])
            .current_dir(root)
            .output()
            .unwrap();

        let findings = check_conventional_commits(root);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "HIST001");
    }

    #[test]
    fn test_giant_commits_none() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        init_git_repo(root);

        // Create a small commit
        std::fs::write(root.join("file.txt"), "content").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(root)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "feat: small change"])
            .current_dir(root)
            .output()
            .unwrap();

        let findings = check_giant_commits(root);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_unsigned_commits() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        init_git_repo(root);

        std::fs::write(root.join("file.txt"), "content").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(root)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "feat: unsigned commit"])
            .current_dir(root)
            .output()
            .unwrap();

        let findings = check_unsigned_commits(root);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "HIST003");
    }

    #[test]
    fn test_thresholds() {
        assert_eq!(COMMITS_TO_ANALYZE, 100);
        assert_eq!(GIANT_COMMIT_THRESHOLD, 50);
    }
}
