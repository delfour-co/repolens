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
}
