//! File-related rules
//!
//! This module provides rules for checking repository files, including:
//! - Large files that should use Git LFS
//! - .gitignore configuration and recommended entries
//! - Temporary files that shouldn't be committed

use crate::config::Config;
use crate::error::RepoLensError;
use crate::rules::engine::RuleCategory;
use crate::rules::results::{Finding, Severity};
use crate::scanner::Scanner;
use crate::utils::{detect_languages, get_gitignore_entries_with_descriptions};

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
    async fn run(&self, scanner: &Scanner, config: &Config) -> Result<Vec<Finding>, RepoLensError> {
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
async fn check_large_files(scanner: &Scanner) -> Result<Vec<Finding>, RepoLensError> {
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
async fn check_gitignore(scanner: &Scanner) -> Result<Vec<Finding>, RepoLensError> {
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

    // Check for recommended entries based on detected languages
    let gitignore_content = scanner.read_file(".gitignore").unwrap_or_else(|e| {
        tracing::warn!("Failed to read .gitignore: {}", e);
        String::new()
    });

    // Detect languages present in the repository
    let languages = detect_languages(scanner);

    // Get recommended entries for detected languages
    let recommended_entries = get_gitignore_entries_with_descriptions(&languages);

    for (pattern, description) in recommended_entries {
        // Check if pattern already exists (handle various formats)
        let pattern_clean = pattern.trim_end_matches('/');
        let pattern_variants = [
            pattern.as_str(),
            &format!("/{}", pattern),
            &format!("{}/", pattern),
            pattern_clean,
            &format!("/{}", pattern_clean),
            &format!("{}/", pattern_clean),
        ];

        let exists = gitignore_content.lines().any(|line| {
            let line = line.trim();
            let line_clean = line.trim_end_matches('/');
            pattern_variants
                .iter()
                .any(|p| line == *p || line_clean == pattern_clean)
        });

        if !exists {
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
async fn check_temp_files(scanner: &Scanner) -> Result<Vec<Finding>, RepoLensError> {
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

    #[tokio::test]
    async fn test_check_gitignore_rust_project_no_node_modules() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let gitignore = root.join(".gitignore");
        let cargo_toml = root.join("Cargo.toml");

        // Create a Rust project
        fs::write(
            &cargo_toml,
            "[package]\nname = \"test\"\nversion = \"0.1.0\"",
        )
        .unwrap();
        fs::write(&gitignore, ".env\n*.key\n").unwrap();

        let scanner = Scanner::new(root.to_path_buf());
        let findings = check_gitignore(&scanner).await.unwrap();

        // Should suggest target/ for Rust
        let target_finding = findings
            .iter()
            .find(|f| f.rule_id == "FILE003" && f.message.contains("target/"));
        assert!(
            target_finding.is_some(),
            "Should suggest target/ for Rust projects"
        );

        // Should NOT suggest node_modules for Rust projects
        let node_modules_finding = findings
            .iter()
            .find(|f| f.rule_id == "FILE003" && f.message.contains("node_modules"));
        assert!(
            node_modules_finding.is_none(),
            "Should NOT suggest node_modules for Rust projects"
        );
    }

    #[tokio::test]
    async fn test_check_gitignore_javascript_project_suggests_node_modules() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let gitignore = root.join(".gitignore");
        let package_json = root.join("package.json");

        // Create a JavaScript project
        fs::write(
            &package_json,
            "{\"name\": \"test\", \"version\": \"1.0.0\"}",
        )
        .unwrap();
        fs::write(&gitignore, ".env\n*.key\n").unwrap();

        let scanner = Scanner::new(root.to_path_buf());
        let findings = check_gitignore(&scanner).await.unwrap();

        // Should suggest node_modules/ for JavaScript
        let node_modules_finding = findings
            .iter()
            .find(|f| f.rule_id == "FILE003" && f.message.contains("node_modules"));
        assert!(
            node_modules_finding.is_some(),
            "Should suggest node_modules/ for JavaScript projects"
        );

        // Should NOT suggest target/ for JavaScript projects
        let target_finding = findings
            .iter()
            .find(|f| f.rule_id == "FILE003" && f.message.contains("target/"));
        assert!(
            target_finding.is_none(),
            "Should NOT suggest target/ for JavaScript projects"
        );
    }

    #[tokio::test]
    async fn test_check_gitignore_universal_entries_always_suggested() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let gitignore = root.join(".gitignore");

        // Create empty .gitignore
        fs::write(&gitignore, "").unwrap();

        let scanner = Scanner::new(root.to_path_buf());
        let findings = check_gitignore(&scanner).await.unwrap();

        // Should suggest universal entries
        let env_finding = findings
            .iter()
            .find(|f| f.rule_id == "FILE003" && f.message.contains(".env"));
        assert!(
            env_finding.is_some(),
            "Should suggest .env (universal entry)"
        );

        let key_finding = findings
            .iter()
            .find(|f| f.rule_id == "FILE003" && f.message.contains("*.key"));
        assert!(
            key_finding.is_some(),
            "Should suggest *.key (universal entry)"
        );
    }

    #[tokio::test]
    async fn test_check_gitignore_python_project_suggests_python_entries() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let gitignore = root.join(".gitignore");
        let requirements_txt = root.join("requirements.txt");

        // Create a Python project
        fs::write(&requirements_txt, "requests==2.28.0\n").unwrap();
        fs::write(&gitignore, ".env\n").unwrap();

        let scanner = Scanner::new(root.to_path_buf());
        let findings = check_gitignore(&scanner).await.unwrap();

        // Should suggest Python-specific entries
        let pycache_finding = findings
            .iter()
            .find(|f| f.rule_id == "FILE003" && f.message.contains("__pycache__"));
        assert!(
            pycache_finding.is_some(),
            "Should suggest __pycache__/ for Python projects"
        );

        // Should NOT suggest node_modules
        let node_modules_finding = findings
            .iter()
            .find(|f| f.rule_id == "FILE003" && f.message.contains("node_modules"));
        assert!(
            node_modules_finding.is_none(),
            "Should NOT suggest node_modules for Python projects"
        );
    }
}
