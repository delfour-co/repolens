//! Code quality rules
//!
//! This module provides rules for checking code quality aspects, including:
//! - Test files and directories
//! - Linting configuration
//! - Editor configuration files

use anyhow::Result;

use crate::config::Config;
use crate::rules::engine::RuleCategory;
use crate::rules::results::{Finding, Severity};
use crate::scanner::Scanner;

/// Rules for checking code quality
pub struct QualityRules;

#[async_trait::async_trait]
impl RuleCategory for QualityRules {
    /// Get the category name
    fn name(&self) -> &'static str {
        "quality"
    }

    /// Run all quality-related rules
    ///
    /// # Arguments
    ///
    /// * `scanner` - The scanner to access repository files
    /// * `config` - The configuration with enabled rules
    ///
    /// # Returns
    ///
    /// A vector of findings for quality issues
    async fn run(&self, scanner: &Scanner, config: &Config) -> Result<Vec<Finding>> {
        let mut findings = Vec::new();

        // Check for tests
        if config.is_rule_enabled("quality/tests") {
            findings.extend(check_tests(scanner).await?);
        }

        // Check for linting configuration
        if config.is_rule_enabled("quality/linting") {
            findings.extend(check_linting(scanner).await?);
        }

        // Check for editor configuration
        if config.is_rule_enabled("files/editorconfig") {
            findings.extend(check_editorconfig(scanner).await?);
        }

        Ok(findings)
    }
}

/// Check for test files and test configuration
///
/// Verifies that the repository has tests and appropriate test configuration.
///
/// # Arguments
///
/// * `scanner` - The scanner to access repository files
///
/// # Returns
///
/// A vector of findings for test-related issues
async fn check_tests(scanner: &Scanner) -> Result<Vec<Finding>> {
    let mut findings = Vec::new();

    // Check for test directories
    let test_dirs = ["test", "tests", "__tests__", "spec", "specs"];
    let has_test_dir = test_dirs.iter().any(|d| scanner.directory_exists(d));

    // Check for test files
    let test_file_patterns = ["*.test.*", "*.spec.*", "*_test.*", "*Test.*"];
    let has_test_files = test_file_patterns
        .iter()
        .any(|p| !scanner.files_matching_pattern(p).is_empty());

    if !has_test_dir && !has_test_files {
        findings.push(
            Finding::new(
                "QUALITY001",
                "quality",
                Severity::Info,
                "No tests detected",
            )
            .with_description(
                "Tests are important for ensuring code quality and catching regressions."
            )
            .with_remediation(
                "Add tests to your project. Consider using a testing framework appropriate for your language."
            )
        );
    }

    // Check if package.json has test script
    if scanner.file_exists("package.json") {
        if let Ok(content) = scanner.read_file("package.json") {
            if !content.contains(r#""test""#) || content.contains(r#""test": "echo"#) {
                findings.push(
                    Finding::new(
                        "QUALITY002",
                        "quality",
                        Severity::Info,
                        "No test script defined in package.json",
                    )
                    .with_description(
                        "A 'test' script in package.json enables running tests with 'npm test'.",
                    ),
                );
            }
        }
    }

    Ok(findings)
}

/// Check for linting configuration files
///
/// Verifies that appropriate linting tools are configured based on
/// the project type (JavaScript, Python, Ruby, Go, Rust, etc.).
///
/// # Arguments
///
/// * `scanner` - The scanner to access repository files
///
/// # Returns
///
/// A vector of findings for missing linting configuration
async fn check_linting(scanner: &Scanner) -> Result<Vec<Finding>> {
    let mut findings = Vec::new();

    // Linting config files by language/tool
    let linting_configs = [
        // JavaScript/TypeScript
        (".eslintrc", "ESLint"),
        (".eslintrc.js", "ESLint"),
        (".eslintrc.json", "ESLint"),
        (".eslintrc.yml", "ESLint"),
        ("eslint.config.js", "ESLint"),
        ("biome.json", "Biome"),
        // Formatting
        (".prettierrc", "Prettier"),
        (".prettierrc.js", "Prettier"),
        (".prettierrc.json", "Prettier"),
        // Python
        ("pyproject.toml", "Python tooling"),
        (".flake8", "Flake8"),
        ("setup.cfg", "Python tooling"),
        (".pylintrc", "Pylint"),
        ("ruff.toml", "Ruff"),
        // Ruby
        (".rubocop.yml", "RuboCop"),
        // Go
        (".golangci.yml", "golangci-lint"),
        (".golangci.yaml", "golangci-lint"),
        // Rust
        ("rustfmt.toml", "rustfmt"),
        (".rustfmt.toml", "rustfmt"),
        ("clippy.toml", "Clippy"),
    ];

    // Detect project type
    let is_js_project = scanner.file_exists("package.json");
    let is_python_project =
        scanner.file_exists("pyproject.toml") || scanner.file_exists("requirements.txt");
    let is_ruby_project = scanner.file_exists("Gemfile");
    let is_go_project = scanner.file_exists("go.mod");
    let is_rust_project = scanner.file_exists("Cargo.toml");

    let has_linting = linting_configs.iter().any(|(f, _)| scanner.file_exists(f));

    if !has_linting
        && (is_js_project
            || is_python_project
            || is_ruby_project
            || is_go_project
            || is_rust_project)
    {
        let suggestion = if is_js_project {
            "ESLint for linting and Prettier for formatting"
        } else if is_python_project {
            "Ruff or Flake8 for linting"
        } else if is_ruby_project {
            "RuboCop for linting"
        } else if is_go_project {
            "golangci-lint for linting"
        } else {
            "Clippy for linting and rustfmt for formatting"
        };

        findings.push(
            Finding::new(
                "QUALITY003",
                "quality",
                Severity::Info,
                "No linting configuration detected",
            )
            .with_description(
                "Linting tools help maintain consistent code style and catch potential issues.",
            )
            .with_remediation(format!("Consider adding {} to your project.", suggestion)),
        );
    }

    Ok(findings)
}

/// Check for .editorconfig file
///
/// Verifies that an .editorconfig file exists to maintain consistent
/// coding styles across editors and IDEs.
///
/// # Arguments
///
/// * `scanner` - The scanner to access repository files
///
/// # Returns
///
/// A vector of findings for missing .editorconfig
async fn check_editorconfig(scanner: &Scanner) -> Result<Vec<Finding>> {
    let mut findings = Vec::new();

    if !scanner.file_exists(".editorconfig") {
        findings.push(
            Finding::new(
                "QUALITY004",
                "quality",
                Severity::Info,
                ".editorconfig file is missing",
            )
            .with_description(
                "EditorConfig helps maintain consistent coding styles across different editors and IDEs."
            )
            .with_remediation(
                "Create a .editorconfig file to define coding style preferences."
            )
        );
    }

    Ok(findings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::Scanner;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_check_tests_no_tests_detected() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        let scanner = Scanner::new(root.to_path_buf());
        let findings = check_tests(&scanner).await.unwrap();

        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| f.rule_id == "QUALITY001"));
    }

    #[tokio::test]
    async fn test_check_tests_package_json_no_test_script() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let package_json = root.join("package.json");

        // Create package.json without test script or with echo test
        fs::write(
            &package_json,
            r#"{"name": "test", "version": "1.0.0", "scripts": {"test": "echo \"No tests\""}}"#,
        )
        .unwrap();

        let scanner = Scanner::new(root.to_path_buf());
        let findings = check_tests(&scanner).await.unwrap();

        // Should find QUALITY002 because test script is just "echo"
        assert!(findings.iter().any(|f| f.rule_id == "QUALITY002"));
    }

    #[tokio::test]
    async fn test_check_linting_no_config() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let package_json = root.join("package.json");

        fs::write(&package_json, r#"{"name": "test"}"#).unwrap();

        let scanner = Scanner::new(root.to_path_buf());
        let findings = check_linting(&scanner).await.unwrap();

        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| f.rule_id == "QUALITY003"));
    }

    #[tokio::test]
    async fn test_check_editorconfig_missing() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        let scanner = Scanner::new(root.to_path_buf());
        let findings = check_editorconfig(&scanner).await.unwrap();

        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| f.rule_id == "QUALITY004"));
    }
}
