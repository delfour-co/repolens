//! File-related rules

use anyhow::Result;

use crate::config::Config;
use crate::rules::engine::RuleCategory;
use crate::rules::results::{Finding, Severity};
use crate::scanner::Scanner;

pub struct FilesRules;

#[async_trait::async_trait]
impl RuleCategory for FilesRules {
    fn name(&self) -> &'static str {
        "files"
    }

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
                "Large files can slow down repository operations and increase clone times."
            )
            .with_remediation(
                "Consider using Git LFS (Large File Storage) for binary or large files."
            )
        );
    }

    Ok(findings)
}

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
                "A .gitignore file helps prevent accidentally committing unwanted files."
            )
            .with_remediation(
                "Create a .gitignore file with appropriate patterns for your project type."
            )
        );
        return Ok(findings);
    }

    // Check for recommended entries
    let gitignore_content = scanner.read_file(".gitignore").unwrap_or_default();
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
                    pattern, description.to_lowercase()
                ))
            );
        }
    }

    Ok(findings)
}

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
                .with_description(
                    "Temporary files should not be committed to version control."
                )
                .with_remediation(
                    "Remove the file and add the pattern to .gitignore."
                )
            );
        }
    }

    Ok(findings)
}
