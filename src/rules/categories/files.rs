//! File-related rules
//!
//! This module provides rules for checking repository files, including:
//! - Large files that should use Git LFS
//! - .gitignore configuration and recommended entries
//! - Temporary files that shouldn't be committed

use anyhow::Result;

use crate::config::Config;
use crate::rules::engine::RuleCategory;
use crate::rules::results::{Finding, Severity};
use crate::scanner::Scanner;

/// Rules for checking repository files
pub struct FilesRules;

#[async_trait::async_trait]
impl RuleCategory for FilesRules {
    /// Get the category name
    fn name(&self) -> &'static str {
        "files"
    }

    /// Run all file-related rules
    ///
    /// # Arguments
    ///
    /// * `scanner` - The scanner to access repository files
    /// * `config` - The configuration with enabled rules
    ///
    /// # Returns
    ///
    /// A vector of findings for file-related issues
    async fn run(&self, scanner: &Scanner, config: &Config) -> Result<Vec<Finding>> {
        let mut findings = Vec::new();

        // Check for large files
        if config.is_rule_enabled("files/large") {
            findings.extend(check_large_files(scanner).await?);
        }

        // Check .gitignore
        if config.is_rule_enabled("files/gitignore") {
            findings.extend(check_gitignore(scanner).await?);
        }

        // Check for temporary files
        if config.is_rule_enabled("files/temp") {
            findings.extend(check_temp_files(scanner).await?);
        }

        Ok(findings)
    }
}

/// Check for files larger than the recommended threshold
///
/// Large files can slow down repository operations and should use Git LFS.
///
/// # Arguments
///
/// * `scanner` - The scanner to access repository files
///
/// # Returns
///
/// A vector of findings for large files
async fn check_large_files(scanner: &Scanner) -> Result<Vec<Finding>> {
    let mut findings = Vec::new();

    // 10MB threshold
    const LARGE_FILE_THRESHOLD: u64 = 10 * 1024 * 1024;

    for file in scanner.files_larger_than(LARGE_FILE_THRESHOLD) {
        let size_mb = file.size as f64 / 1024.0 / 1024.0;

        findings.push(
            Finding::new(
                "FILE001",
                "files",
                Severity::Warning,
                format!("Large file detected ({:.1} MB)", size_mb),
            )
            .with_location(&file.path)
            .with_description(
                "Large files can slow down repository operations and increase clone times.",
            )
            .with_remediation(
                "Consider using Git LFS (Large File Storage) for binary or large files.",
            ),
        );
    }

    Ok(findings)
}

/// Check .gitignore file existence and recommended entries
///
/// Verifies that .gitignore exists and contains recommended patterns
/// to prevent committing unwanted files.
///
/// # Arguments
///
/// * `scanner` - The scanner to access repository files
///
/// # Returns
///
/// A vector of findings for .gitignore issues
async fn check_gitignore(scanner: &Scanner) -> Result<Vec<Finding>> {
    let mut findings = Vec::new();

    // Check if .gitignore exists
    if !scanner.file_exists(".gitignore") {
        findings.push(
            Finding::new(
                "FILE002",
                "files",
                Severity::Warning,
                ".gitignore file is missing",
            )
            .with_description(
                "A .gitignore file helps prevent accidentally committing unwanted files.",
            )
            .with_remediation(
                "Create a .gitignore file with appropriate patterns for your project type.",
            ),
        );
        return Ok(findings);
    }

    // Check for recommended entries
    let gitignore_content = scanner.read_file(".gitignore").unwrap_or_else(|e| {
        tracing::warn!("Failed to read .gitignore: {}", e);
        String::new()
    });
    let recommended_entries = [
        (".env", "Environment files"),
        ("*.key", "Private keys"),
        ("*.pem", "Certificates"),
        ("node_modules", "Node.js dependencies"),
        (".DS_Store", "macOS metadata"),
    ];

    for (pattern, description) in recommended_entries {
        if !gitignore_content.contains(pattern) {
            findings.push(
                Finding::new(
                    "FILE003",
                    "files",
                    Severity::Info,
                    format!(".gitignore missing recommended entry: {}", pattern),
                )
                .with_description(format!(
                    "Adding '{}' to .gitignore helps prevent committing {}.",
                    pattern,
                    description.to_lowercase()
                )),
            );
        }
    }

    Ok(findings)
}

/// Check for temporary files that shouldn't be committed
///
/// Detects common temporary file patterns like .log, .tmp, .swp, etc.
///
/// # Arguments
///
/// * `scanner` - The scanner to access repository files
///
/// # Returns
///
/// A vector of findings for temporary files
async fn check_temp_files(scanner: &Scanner) -> Result<Vec<Finding>> {
    let mut findings = Vec::new();

    let temp_patterns = ["*.log", "*.tmp", "*.temp", "*~", "*.swp", "*.swo", "*.bak"];

    for pattern in temp_patterns {
        for file in scanner.files_matching_pattern(pattern) {
            findings.push(
                Finding::new(
                    "FILE004",
                    "files",
                    Severity::Warning,
                    "Temporary file found in repository",
                )
                .with_location(&file.path)
                .with_description("Temporary files should not be committed to version control.")
                .with_remediation("Remove the file and add the pattern to .gitignore."),
            );
        }
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
    async fn test_check_large_files_detects_large_file() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let large_file = root.join("large.bin");

        let large_content = vec![0u8; 11 * 1024 * 1024];
        fs::write(&large_file, large_content).unwrap();

        let scanner = Scanner::new(root.to_path_buf());
        let findings = check_large_files(&scanner).await.unwrap();

        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| f.rule_id == "FILE001"));
    }

    #[tokio::test]
    async fn test_check_gitignore_missing() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        let scanner = Scanner::new(root.to_path_buf());
        let findings = check_gitignore(&scanner).await.unwrap();

        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| f.rule_id == "FILE002"));
    }

    #[tokio::test]
    async fn test_check_gitignore_missing_recommended_entries() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let gitignore = root.join(".gitignore");

        fs::write(&gitignore, "node_modules/").unwrap();

        let scanner = Scanner::new(root.to_path_buf());
        let findings = check_gitignore(&scanner).await.unwrap();

        assert!(findings.iter().any(|f| f.rule_id == "FILE003"));
    }

    #[tokio::test]
    async fn test_check_temp_files_detects_tmp() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let tmp_file = root.join("temp.tmp");

        fs::write(&tmp_file, "temporary content").unwrap();

        let scanner = Scanner::new(root.to_path_buf());
        let findings = check_temp_files(&scanner).await.unwrap();

        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| f.rule_id == "FILE004"));
    }
}
