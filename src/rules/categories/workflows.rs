//! GitHub Actions workflow rules
//!
//! This module provides rules for checking GitHub Actions workflows, including:
//! - Hardcoded secrets in workflow files
//! - Explicit permissions configuration
//! - Pinned action versions for security

use crate::error::RepoLensError;
use regex::Regex;

use crate::config::Config;
use crate::rules::engine::RuleCategory;
use crate::rules::results::{Finding, Severity};
use crate::scanner::Scanner;

/// Rules for checking GitHub Actions workflows
pub struct WorkflowsRules;

#[async_trait::async_trait]
impl RuleCategory for WorkflowsRules {
    /// Get the category name
    fn name(&self) -> &'static str {
        "workflows"
    }

    /// Run all workflow-related rules
    ///
    /// # Arguments
    ///
    /// * `scanner` - The scanner to access repository files
    /// * `config` - The configuration with enabled rules
    ///
    /// # Returns
    ///
    /// A vector of findings for workflow issues
    async fn run(&self, scanner: &Scanner, config: &Config) -> Result<Vec<Finding>, RepoLensError> {
        let mut findings = Vec::new();

        // Check for workflows directory
        if !scanner.directory_exists(".github/workflows") {
            return Ok(findings);
        }

        // Check workflow security
        if config.is_rule_enabled("workflows/secrets") {
            findings.extend(check_workflow_secrets(scanner).await?);
        }

        // Check permissions
        if config.is_rule_enabled("workflows/permissions") {
            findings.extend(check_workflow_permissions(scanner).await?);
        }

        // Check pinned actions
        if config.is_rule_enabled("workflows/pinned-actions") {
            findings.extend(check_pinned_actions(scanner, config).await?);
        }

        Ok(findings)
    }
}

/// Check for hardcoded secrets in workflow files
///
/// Detects patterns that suggest hardcoded passwords, tokens, API keys,
/// or secrets in GitHub Actions workflow files.
///
/// # Arguments
///
/// * `scanner` - The scanner to access repository files
///
/// # Returns
///
/// A vector of findings for hardcoded secrets in workflows
async fn check_workflow_secrets(scanner: &Scanner) -> Result<Vec<Finding>, RepoLensError> {
    let mut findings = Vec::new();

    // Patterns that suggest hardcoded secrets in workflows
    let secret_patterns = [
        (r#"password\s*:\s*['"][^'"]+['"]"#, "hardcoded password"),
        (r#"token\s*:\s*['"][^'"]+['"]"#, "hardcoded token"),
        (r#"api[_-]?key\s*:\s*['"][^'"]+['"]"#, "hardcoded API key"),
        (r#"secret\s*:\s*['"][^'"]+['"]"#, "hardcoded secret"),
    ];

    for file in scanner.files_in_directory(".github/workflows") {
        if !file.path.ends_with(".yml") && !file.path.ends_with(".yaml") {
            continue;
        }

        if let Ok(content) = scanner.read_file(&file.path) {
            for (pattern, description) in &secret_patterns {
                let regex = match Regex::new(pattern) {
                    Ok(r) => r,
                    Err(e) => {
                        tracing::warn!("Invalid regex pattern '{}': {}", pattern, e);
                        continue;
                    }
                };
                if regex.is_match(&content) {
                    // Find line number
                    let line_num = content
                        .lines()
                        .enumerate()
                        .find(|(_, line)| regex.is_match(line))
                        .map(|(i, _)| i + 1)
                        .unwrap_or(0);

                    findings.push(
                        Finding::new(
                            "WF001",
                            "workflows",
                            Severity::Critical,
                            format!("Potential {} in workflow", description),
                        )
                        .with_location(format!("{}:{}", file.path, line_num))
                        .with_description("Secrets should never be hardcoded in workflow files.")
                        .with_remediation(
                            "Use GitHub Secrets (secrets.SECRET_NAME) instead of hardcoded values.",
                        ),
                    );
                }
            }
        }
    }

    Ok(findings)
}

/// Check for explicit permissions in workflow files
///
/// Verifies that workflows define explicit permissions to follow
/// the principle of least privilege.
///
/// # Arguments
///
/// * `scanner` - The scanner to access repository files
///
/// # Returns
///
/// A vector of findings for missing permissions
async fn check_workflow_permissions(scanner: &Scanner) -> Result<Vec<Finding>, RepoLensError> {
    let mut findings = Vec::new();

    for file in scanner.files_in_directory(".github/workflows") {
        if !file.path.ends_with(".yml") && !file.path.ends_with(".yaml") {
            continue;
        }

        if let Ok(content) = scanner.read_file(&file.path) {
            // Check if permissions are defined
            if !content.contains("permissions:") {
                findings.push(
                    Finding::new(
                        "WF002",
                        "workflows",
                        Severity::Warning,
                        "Workflow missing explicit permissions",
                    )
                    .with_location(&file.path)
                    .with_description(
                        "Workflows without explicit permissions use the default permissions, which may be more permissive than necessary."
                    )
                    .with_remediation(
                        "Add a 'permissions:' block to explicitly define the minimum required permissions."
                    )
                );
            }
        }
    }

    Ok(findings)
}

/// Check for pinned action versions
///
/// In strict mode, verifies that actions are pinned to specific versions
/// instead of using @main, @master, or @latest.
///
/// # Arguments
///
/// * `scanner` - The scanner to access repository files
/// * `config` - The configuration (used to check preset)
///
/// # Returns
///
/// A vector of findings for unpinned actions
async fn check_pinned_actions(
    scanner: &Scanner,
    config: &Config,
) -> Result<Vec<Finding>, RepoLensError> {
    let mut findings = Vec::new();

    // Only check in strict mode
    if config.preset != "strict" {
        return Ok(findings);
    }

    let unpinned_patterns = [
        r"uses:\s+\S+@main\b",
        r"uses:\s+\S+@master\b",
        r"uses:\s+\S+@latest\b",
    ];

    for file in scanner.files_in_directory(".github/workflows") {
        if !file.path.ends_with(".yml") && !file.path.ends_with(".yaml") {
            continue;
        }

        if let Ok(content) = scanner.read_file(&file.path) {
            for pattern in &unpinned_patterns {
                let regex = match Regex::new(pattern) {
                    Ok(r) => r,
                    Err(e) => {
                        tracing::warn!("Invalid regex pattern '{}': {}", pattern, e);
                        continue;
                    }
                };
                for (line_num, line) in content.lines().enumerate() {
                    if regex.is_match(line) {
                        findings.push(
                            Finding::new(
                                "WF003",
                                "workflows",
                                Severity::Warning,
                                "Workflow uses unpinned action reference",
                            )
                            .with_location(format!("{}:{}", file.path, line_num + 1))
                            .with_description(
                                "Using @main, @master, or @latest for actions can introduce breaking changes or security vulnerabilities."
                            )
                            .with_remediation(
                                "Pin actions to a specific version tag (e.g., @v4) or commit SHA for maximum security."
                            )
                        );
                    }
                }
            }
        }
    }

    Ok(findings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::scanner::Scanner;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_check_workflow_secrets_detects_hardcoded_password() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let workflows_dir = root.join(".github").join("workflows");
        fs::create_dir_all(&workflows_dir).unwrap();

        let workflow_file = workflows_dir.join("ci.yml");
        fs::write(
            &workflow_file,
            "name: CI\non: push\njobs:\n  test:\n    password: 'secret123'",
        )
        .unwrap();

        let scanner = Scanner::new(root.to_path_buf());
        let findings = check_workflow_secrets(&scanner).await.unwrap();

        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| f.rule_id == "WF001"));
    }

    #[tokio::test]
    async fn test_check_workflow_permissions_missing() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let workflows_dir = root.join(".github").join("workflows");
        fs::create_dir_all(&workflows_dir).unwrap();

        let workflow_file = workflows_dir.join("ci.yml");
        fs::write(
            &workflow_file,
            "name: CI\non: push\njobs:\n  test:\n    runs-on: ubuntu-latest",
        )
        .unwrap();

        let scanner = Scanner::new(root.to_path_buf());
        let findings = check_workflow_permissions(&scanner).await.unwrap();

        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| f.rule_id == "WF002"));
    }

    #[tokio::test]
    async fn test_check_pinned_actions_detects_unpinned() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let workflows_dir = root.join(".github").join("workflows");
        fs::create_dir_all(&workflows_dir).unwrap();

        let workflow_file = workflows_dir.join("ci.yml");
        fs::write(
            &workflow_file,
            "name: CI\njobs:\n  test:\n    uses: actions/checkout@main",
        )
        .unwrap();

        let scanner = Scanner::new(root.to_path_buf());
        let config = Config {
            preset: "strict".to_string(),
            ..Default::default()
        };
        let findings = check_pinned_actions(&scanner, &config).await.unwrap();

        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| f.rule_id == "WF003"));
    }
}
