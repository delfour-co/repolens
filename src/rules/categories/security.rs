//! Security rules
//!
//! This module provides rules for checking security-related aspects, including:
//! - CODEOWNERS file for code review requirements
//! - Dependency lock files for reproducible builds
//! - Runtime version files for consistent environments

use crate::error::RepoLensError;

use crate::config::Config;
use crate::rules::engine::RuleCategory;
use crate::rules::results::{Finding, Severity};
use crate::scanner::Scanner;

/// Rules for checking security-related aspects
pub struct SecurityRules;

#[async_trait::async_trait]
impl RuleCategory for SecurityRules {
    /// Get the category name
    fn name(&self) -> &'static str {
        "security"
    }

    /// Run all security-related rules
    ///
    /// # Arguments
    ///
    /// * `scanner` - The scanner to access repository files
    /// * `config` - The configuration with enabled rules
    ///
    /// # Returns
    ///
    /// A vector of findings for security issues
    async fn run(&self, scanner: &Scanner, config: &Config) -> Result<Vec<Finding>, RepoLensError> {
        let mut findings = Vec::new();

        // Check CODEOWNERS
        if config.is_rule_enabled("security/codeowners") {
            findings.extend(check_codeowners(scanner, config).await?);
        }

        // Check for dependency files
        if config.is_rule_enabled("security/dependencies") {
            findings.extend(check_dependencies(scanner).await?);
        }

        // Check for branch protection configuration
        if config.is_rule_enabled("security/branch-protection") {
            findings.extend(check_branch_protection(scanner).await?);
        }

        Ok(findings)
    }
}

/// Check for CODEOWNERS file
///
/// Verifies that a CODEOWNERS file exists. Required for enterprise preset,
/// recommended for strict preset.
///
/// # Arguments
///
/// * `scanner` - The scanner to access repository files
/// * `config` - The configuration (used to determine severity)
///
/// # Returns
///
/// A vector of findings for CODEOWNERS issues
async fn check_codeowners(
    scanner: &Scanner,
    config: &Config,
) -> Result<Vec<Finding>, RepoLensError> {
    let mut findings = Vec::new();

    let codeowners_files = ["CODEOWNERS", ".github/CODEOWNERS", "docs/CODEOWNERS"];
    let has_codeowners = codeowners_files.iter().any(|f| scanner.file_exists(f));

    // CODEOWNERS is required for enterprise, recommended for strict
    let severity = if config.preset == "enterprise" {
        Severity::Critical
    } else {
        Severity::Info
    };

    if !has_codeowners {
        findings.push(
            Finding::new(
                "SECURITY001",
                "security",
                severity,
                "CODEOWNERS file is missing",
            )
            .with_description(
                "A CODEOWNERS file automatically assigns reviewers to pull requests based on file paths."
            )
            .with_remediation(
                "Create a CODEOWNERS file in .github/ to define code ownership and review requirements."
            )
        );
    }

    Ok(findings)
}

/// Check for dependency lock files and version files
///
/// Verifies that lock files exist for reproducible builds and that
/// runtime version files are specified for consistent environments.
///
/// # Arguments
///
/// * `scanner` - The scanner to access repository files
///
/// # Returns
///
/// A vector of findings for dependency-related issues
async fn check_dependencies(scanner: &Scanner) -> Result<Vec<Finding>, RepoLensError> {
    let mut findings = Vec::new();

    // Check for lock files (indicates dependency management)
    let _lock_files = [
        ("package-lock.json", "npm"),
        ("yarn.lock", "Yarn"),
        ("pnpm-lock.yaml", "pnpm"),
        ("Cargo.lock", "Cargo"),
        ("Gemfile.lock", "Bundler"),
        ("poetry.lock", "Poetry"),
        ("Pipfile.lock", "Pipenv"),
        ("composer.lock", "Composer"),
        ("go.sum", "Go modules"),
    ];

    let package_files = [
        ("package.json", "package-lock.json"),
        ("Cargo.toml", "Cargo.lock"),
        ("Gemfile", "Gemfile.lock"),
        ("pyproject.toml", "poetry.lock"),
        ("Pipfile", "Pipfile.lock"),
        ("composer.json", "composer.lock"),
        ("go.mod", "go.sum"),
    ];

    for (package_file, lock_file) in package_files {
        if scanner.file_exists(package_file) && !scanner.file_exists(lock_file) {
            findings.push(
                Finding::new(
                    "SECURITY002",
                    "security",
                    Severity::Warning,
                    format!("Lock file {} is missing", lock_file),
                )
                .with_description(
                    "Lock files ensure reproducible builds and protect against supply chain attacks."
                )
                .with_remediation(
                    "Generate the lock file by running your package manager's install command."
                )
            );
        }
    }

    // Check for .nvmrc or similar version files
    let version_managers = [
        (".nvmrc", "Node.js version"),
        (".node-version", "Node.js version"),
        (".python-version", "Python version"),
        (".ruby-version", "Ruby version"),
        ("rust-toolchain.toml", "Rust toolchain"),
    ];

    let has_any_version_file = version_managers.iter().any(|(f, _)| scanner.file_exists(f));

    // Detect project type
    let is_node = scanner.file_exists("package.json");
    let is_python =
        scanner.file_exists("pyproject.toml") || scanner.file_exists("requirements.txt");
    let is_ruby = scanner.file_exists("Gemfile");
    let is_rust = scanner.file_exists("Cargo.toml");

    if !has_any_version_file && (is_node || is_python || is_ruby || is_rust) {
        findings.push(
            Finding::new(
                "SECURITY003",
                "security",
                Severity::Info,
                "No runtime version file found",
            )
            .with_description(
                "Specifying runtime versions (e.g., .nvmrc, .python-version) ensures consistent development environments."
            )
        );
    }

    Ok(findings)
}

/// Check for branch protection configuration via .github/settings.yml
///
/// Verifies that branch protection is configured using the probot/settings app
/// configuration file. This is a common way to manage branch protection as code.
///
/// SEC007: .github/settings.yml absent (info)
/// SEC008: No branch protection rules in settings.yml (warning)
/// SEC009: required_pull_request_reviews not configured (warning)
/// SEC010: required_status_checks not configured (warning)
///
/// # Arguments
///
/// * `scanner` - The scanner to access repository files
///
/// # Returns
///
/// A vector of findings for branch protection issues
async fn check_branch_protection(scanner: &Scanner) -> Result<Vec<Finding>, RepoLensError> {
    let mut findings = Vec::new();

    // Check if .github/settings.yml exists
    let settings_path = ".github/settings.yml";
    if !scanner.file_exists(settings_path) {
        findings.push(
            Finding::new(
                "SEC007",
                "security",
                Severity::Info,
                "GitHub settings file (.github/settings.yml) is absent",
            )
            .with_description(
                "The .github/settings.yml file allows you to configure repository settings, \
                 including branch protection rules, as code using the probot/settings app."
            )
            .with_remediation(
                "Consider adding a .github/settings.yml file to manage repository settings as code. \
                 See https://probot.github.io/apps/settings/ for more information."
            )
        );
        return Ok(findings);
    }

    // Read and parse the settings.yml file
    let content = match scanner.read_file(settings_path) {
        Ok(c) => c,
        Err(_) => return Ok(findings),
    };

    let settings: serde_yaml::Value = match serde_yaml::from_str(&content) {
        Ok(v) => v,
        Err(_) => {
            // Invalid YAML, skip further checks
            return Ok(findings);
        }
    };

    // Check for branches configuration
    let branches = settings.get("branches");
    if branches.is_none() {
        findings.push(
            Finding::new(
                "SEC008",
                "security",
                Severity::Warning,
                "No branch protection rules defined in settings.yml",
            )
            .with_location(settings_path)
            .with_description(
                "Branch protection rules help prevent accidental force pushes, \
                 require code reviews, and enforce status checks before merging.",
            )
            .with_remediation(
                "Add a 'branches:' section to your .github/settings.yml to configure \
                 branch protection for important branches like main/master.",
            ),
        );
        return Ok(findings);
    }

    // Check if branches is an array and has protection rules
    let branches_arr = match branches.and_then(|b| b.as_sequence()) {
        Some(arr) => arr,
        None => return Ok(findings),
    };

    let mut has_pr_reviews = false;
    let mut has_status_checks = false;

    for branch in branches_arr {
        // Check protection settings
        if let Some(protection) = branch.get("protection") {
            // Check for required_pull_request_reviews
            if protection.get("required_pull_request_reviews").is_some() {
                has_pr_reviews = true;
            }

            // Check for required_status_checks
            if protection.get("required_status_checks").is_some() {
                has_status_checks = true;
            }
        }
    }

    if !has_pr_reviews {
        findings.push(
            Finding::new(
                "SEC009",
                "security",
                Severity::Warning,
                "required_pull_request_reviews not configured in branch protection",
            )
            .with_location(settings_path)
            .with_description(
                "Requiring pull request reviews ensures that changes are reviewed \
                 by at least one other team member before merging.",
            )
            .with_remediation(
                "Add 'required_pull_request_reviews' to your branch protection settings in \
                 .github/settings.yml. Example:\n\
                 branches:\n\
                   - name: main\n\
                     protection:\n\
                       required_pull_request_reviews:\n\
                         required_approving_review_count: 1",
            ),
        );
    }

    if !has_status_checks {
        findings.push(
            Finding::new(
                "SEC010",
                "security",
                Severity::Warning,
                "required_status_checks not configured in branch protection",
            )
            .with_location(settings_path)
            .with_description(
                "Requiring status checks ensures that CI/CD pipelines pass \
                 before changes can be merged.",
            )
            .with_remediation(
                "Add 'required_status_checks' to your branch protection settings in \
                 .github/settings.yml. Example:\n\
                 branches:\n\
                   - name: main\n\
                     protection:\n\
                       required_status_checks:\n\
                         strict: true\n\
                         contexts:\n\
                           - ci",
            ),
        );
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
    async fn test_check_codeowners_missing_enterprise() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        let scanner = Scanner::new(root.to_path_buf());
        let config = Config {
            preset: "enterprise".to_string(),
            ..Default::default()
        };
        let findings = check_codeowners(&scanner, &config).await.unwrap();

        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| f.rule_id == "SECURITY001"));
        assert!(findings
            .iter()
            .any(|f| f.severity == crate::rules::results::Severity::Critical));
    }

    #[tokio::test]
    async fn test_check_codeowners_missing_strict() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        let scanner = Scanner::new(root.to_path_buf());
        let config = Config {
            preset: "strict".to_string(),
            ..Default::default()
        };
        let findings = check_codeowners(&scanner, &config).await.unwrap();

        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| f.rule_id == "SECURITY001"));
    }

    #[tokio::test]
    async fn test_check_dependencies_missing_lock_file() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let package_json = root.join("package.json");

        fs::write(&package_json, r#"{"name": "test"}"#).unwrap();

        let scanner = Scanner::new(root.to_path_buf());
        let findings = check_dependencies(&scanner).await.unwrap();

        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| f.rule_id == "SECURITY002"));
    }

    #[tokio::test]
    async fn test_check_dependencies_no_version_file() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let package_json = root.join("package.json");

        fs::write(&package_json, r#"{"name": "test"}"#).unwrap();

        let scanner = Scanner::new(root.to_path_buf());
        let findings = check_dependencies(&scanner).await.unwrap();

        assert!(findings.iter().any(|f| f.rule_id == "SECURITY003"));
    }

    // ===== Branch Protection Tests (SEC007-010) =====

    #[tokio::test]
    async fn test_check_branch_protection_no_settings_file() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        let scanner = Scanner::new(root.to_path_buf());
        let findings = check_branch_protection(&scanner).await.unwrap();

        assert_eq!(findings.len(), 1);
        assert!(findings.iter().any(|f| f.rule_id == "SEC007"));
        assert!(findings.iter().any(|f| f.severity == Severity::Info));
    }

    #[tokio::test]
    async fn test_check_branch_protection_no_branches_config() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let github_dir = root.join(".github");
        fs::create_dir_all(&github_dir).unwrap();

        fs::write(
            github_dir.join("settings.yml"),
            r#"
repository:
  name: my-repo
  description: A test repo
"#,
        )
        .unwrap();

        let scanner = Scanner::new(root.to_path_buf());
        let findings = check_branch_protection(&scanner).await.unwrap();

        assert_eq!(findings.len(), 1);
        assert!(findings.iter().any(|f| f.rule_id == "SEC008"));
    }

    #[tokio::test]
    async fn test_check_branch_protection_no_pr_reviews() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let github_dir = root.join(".github");
        fs::create_dir_all(&github_dir).unwrap();

        fs::write(
            github_dir.join("settings.yml"),
            r#"
branches:
  - name: main
    protection:
      required_status_checks:
        strict: true
        contexts:
          - ci
"#,
        )
        .unwrap();

        let scanner = Scanner::new(root.to_path_buf());
        let findings = check_branch_protection(&scanner).await.unwrap();

        // Should have SEC009 (no PR reviews) but not SEC010 (has status checks)
        assert!(findings.iter().any(|f| f.rule_id == "SEC009"));
        assert!(!findings.iter().any(|f| f.rule_id == "SEC010"));
    }

    #[tokio::test]
    async fn test_check_branch_protection_no_status_checks() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let github_dir = root.join(".github");
        fs::create_dir_all(&github_dir).unwrap();

        fs::write(
            github_dir.join("settings.yml"),
            r#"
branches:
  - name: main
    protection:
      required_pull_request_reviews:
        required_approving_review_count: 1
"#,
        )
        .unwrap();

        let scanner = Scanner::new(root.to_path_buf());
        let findings = check_branch_protection(&scanner).await.unwrap();

        // Should have SEC010 (no status checks) but not SEC009 (has PR reviews)
        assert!(!findings.iter().any(|f| f.rule_id == "SEC009"));
        assert!(findings.iter().any(|f| f.rule_id == "SEC010"));
    }

    #[tokio::test]
    async fn test_check_branch_protection_both_missing() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let github_dir = root.join(".github");
        fs::create_dir_all(&github_dir).unwrap();

        fs::write(
            github_dir.join("settings.yml"),
            r#"
branches:
  - name: main
    protection:
      enforce_admins: true
"#,
        )
        .unwrap();

        let scanner = Scanner::new(root.to_path_buf());
        let findings = check_branch_protection(&scanner).await.unwrap();

        // Should have both SEC009 and SEC010
        assert!(findings.iter().any(|f| f.rule_id == "SEC009"));
        assert!(findings.iter().any(|f| f.rule_id == "SEC010"));
    }

    #[tokio::test]
    async fn test_check_branch_protection_fully_configured() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let github_dir = root.join(".github");
        fs::create_dir_all(&github_dir).unwrap();

        fs::write(
            github_dir.join("settings.yml"),
            r#"
branches:
  - name: main
    protection:
      required_pull_request_reviews:
        required_approving_review_count: 2
        dismiss_stale_reviews: true
      required_status_checks:
        strict: true
        contexts:
          - ci
          - tests
      enforce_admins: true
"#,
        )
        .unwrap();

        let scanner = Scanner::new(root.to_path_buf());
        let findings = check_branch_protection(&scanner).await.unwrap();

        // No findings when fully configured
        assert!(findings.is_empty());
    }

    #[tokio::test]
    async fn test_check_branch_protection_invalid_yaml() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let github_dir = root.join(".github");
        fs::create_dir_all(&github_dir).unwrap();

        fs::write(github_dir.join("settings.yml"), "invalid: yaml: content: [").unwrap();

        let scanner = Scanner::new(root.to_path_buf());
        let findings = check_branch_protection(&scanner).await.unwrap();

        // Should not crash, just return empty findings for invalid YAML
        assert!(findings.is_empty());
    }

    #[tokio::test]
    async fn test_check_branch_protection_multiple_branches() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let github_dir = root.join(".github");
        fs::create_dir_all(&github_dir).unwrap();

        fs::write(
            github_dir.join("settings.yml"),
            r#"
branches:
  - name: main
    protection:
      required_pull_request_reviews:
        required_approving_review_count: 1
  - name: develop
    protection:
      required_status_checks:
        strict: true
        contexts:
          - ci
"#,
        )
        .unwrap();

        let scanner = Scanner::new(root.to_path_buf());
        let findings = check_branch_protection(&scanner).await.unwrap();

        // Both rules are satisfied across branches
        assert!(findings.is_empty());
    }
}
