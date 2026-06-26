//! CODEOWNERS rules
//!
//! This module provides rules for checking:
//! - CODEOWNERS file presence and validity
//!
//! ## Rules
//!
//! ### CODEOWNERS Rules
//! - CODE001 (info): CODEOWNERS file is missing
//! - CODE002 (warning): CODEOWNERS file has syntax errors

use crate::config::Config;
use crate::error::RepoLensError;
use crate::rules::engine::RuleCategory;
use crate::rules::results::{Finding, Severity};
use crate::scanner::Scanner;
use regex::Regex;

/// Rules for checking CODEOWNERS
pub struct CodeownersRules;

#[async_trait::async_trait]
impl RuleCategory for CodeownersRules {
    fn name(&self) -> &'static str {
        "codeowners"
    }

    async fn run(&self, scanner: &Scanner, config: &Config) -> Result<Vec<Finding>, RepoLensError> {
        let mut findings = Vec::new();

        // CODEOWNERS rules
        if config.is_rule_enabled("codeowners/presence") {
            findings.extend(check_codeowners_presence(scanner, config));
        }

        if config.is_rule_enabled("codeowners/syntax") {
            findings.extend(check_codeowners_syntax(scanner));
        }

        Ok(findings)
    }
}

/// CODEOWNERS file locations (in order of preference)
const CODEOWNERS_PATHS: &[&str] = &["CODEOWNERS", ".github/CODEOWNERS", "docs/CODEOWNERS"];

/// Find CODEOWNERS file if it exists
fn find_codeowners(scanner: &Scanner) -> Option<(String, String)> {
    for path in CODEOWNERS_PATHS {
        if scanner.file_exists(path) {
            if let Ok(content) = scanner.read_file(path) {
                return Some((path.to_string(), content));
            }
        }
    }
    None
}

/// CODE001: Check if CODEOWNERS file exists
fn check_codeowners_presence(scanner: &Scanner, config: &Config) -> Vec<Finding> {
    let mut findings = Vec::new();

    if find_codeowners(scanner).is_none() {
        let severity = if config.preset == "enterprise" {
            Severity::Critical
        } else {
            Severity::Info
        };

        findings.push(
            Finding::new(
                "CODE001",
                "codeowners",
                severity,
                "CODEOWNERS file is missing",
            )
            .with_description(
                "A CODEOWNERS file automatically assigns reviewers to pull requests \
                 based on file paths. This ensures code changes are reviewed by the \
                 appropriate team members.",
            )
            .with_remediation(
                "Create a CODEOWNERS file in .github/, the repository root, or docs/.\n\
                 Example content:\n\
                 # Default owners for everything\n\
                 * @org/team-name\n\n\
                 # Frontend code\n\
                 /src/frontend/ @org/frontend-team\n\n\
                 # Documentation\n\
                 /docs/ @org/docs-team",
            ),
        );
    }

    findings
}

/// CODE002: Check CODEOWNERS syntax
fn check_codeowners_syntax(scanner: &Scanner) -> Vec<Finding> {
    let mut findings = Vec::new();

    let Some((path, content)) = find_codeowners(scanner) else {
        return findings;
    };

    let syntax_errors = validate_codeowners_syntax(&content);

    for (line_num, error) in syntax_errors {
        findings.push(
            Finding::new(
                "CODE002",
                "codeowners",
                Severity::Warning,
                format!("CODEOWNERS syntax error on line {}: {}", line_num, error),
            )
            .with_location(format!("{}:{}", path, line_num))
            .with_description(
                "CODEOWNERS files must follow a specific syntax. Each line should contain \
                 a file pattern followed by one or more owners (GitHub usernames or team names).",
            )
            .with_remediation(
                "Fix the syntax error. Valid formats:\n\
                 - `* @owner` - All files\n\
                 - `/path/ @owner` - Specific directory\n\
                 - `*.js @owner` - File pattern\n\
                 - `# comment` - Comment line",
            ),
        );
    }

    findings
}

/// Validate CODEOWNERS file syntax and return errors with line numbers
fn validate_codeowners_syntax(content: &str) -> Vec<(usize, String)> {
    let mut errors = Vec::new();

    // Pattern for valid owner references: @user, @org/team, or email
    let owner_pattern =
        Regex::new(r"^(@[\w\-\.]+(/[\w\-\.]+)?|[\w\-\.]+@[\w\-\.]+\.\w+)$").unwrap();

    // Pattern for valid file patterns (basic glob check)
    let file_pattern = Regex::new(r"^[/\*\w\.\-\[\]{}!?]+$").unwrap();

    for (line_num, line) in content.lines().enumerate() {
        let line_num = line_num + 1; // 1-indexed
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Split into parts
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.is_empty() {
            continue;
        }

        // First part should be a file pattern
        let pattern = parts[0];

        // Check for basic pattern validity
        if !file_pattern.is_match(pattern) && !pattern.contains('/') && pattern != "*" {
            errors.push((line_num, format!("Invalid file pattern: '{}'", pattern)));
            continue;
        }

        // Must have at least one owner
        if parts.len() < 2 {
            errors.push((line_num, "No owners specified for pattern".to_string()));
            continue;
        }

        // Validate each owner
        for owner in &parts[1..] {
            if !owner_pattern.is_match(owner) {
                errors.push((
                    line_num,
                    format!(
                        "Invalid owner format: '{}'. Must be @username, @org/team, or email",
                        owner
                    ),
                ));
            }
        }
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // ===== CODEOWNERS Presence Tests (CODE001) =====

    #[test]
    fn test_check_codeowners_presence_missing() {
        let temp_dir = TempDir::new().unwrap();
        let scanner = Scanner::new(temp_dir.path().to_path_buf());
        let config = Config::default();

        let findings = check_codeowners_presence(&scanner, &config);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "CODE001");
        assert_eq!(findings[0].severity, Severity::Info);
    }

    #[test]
    fn test_check_codeowners_presence_missing_enterprise() {
        let temp_dir = TempDir::new().unwrap();
        let scanner = Scanner::new(temp_dir.path().to_path_buf());
        let config = Config {
            preset: "enterprise".to_string(),
            ..Default::default()
        };

        let findings = check_codeowners_presence(&scanner, &config);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "CODE001");
        assert_eq!(findings[0].severity, Severity::Critical);
    }

    #[test]
    fn test_check_codeowners_presence_in_root() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("CODEOWNERS"), "* @owner").unwrap();

        let scanner = Scanner::new(temp_dir.path().to_path_buf());
        let config = Config::default();

        let findings = check_codeowners_presence(&scanner, &config);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_check_codeowners_presence_in_github_dir() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join(".github")).unwrap();
        fs::write(temp_dir.path().join(".github/CODEOWNERS"), "* @owner").unwrap();

        let scanner = Scanner::new(temp_dir.path().to_path_buf());
        let config = Config::default();

        let findings = check_codeowners_presence(&scanner, &config);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_check_codeowners_presence_in_docs_dir() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("docs")).unwrap();
        fs::write(temp_dir.path().join("docs/CODEOWNERS"), "* @owner").unwrap();

        let scanner = Scanner::new(temp_dir.path().to_path_buf());
        let config = Config::default();

        let findings = check_codeowners_presence(&scanner, &config);
        assert!(findings.is_empty());
    }

    // ===== CODEOWNERS Syntax Tests (CODE002) =====

    #[test]
    fn test_validate_codeowners_syntax_valid() {
        let content = r#"
# This is a comment
* @global-owner

/src/ @src-team
/docs/*.md @docs-team
*.js @frontend-team
/api/ @org/api-team
/config/ user@example.com
"#;

        let errors = validate_codeowners_syntax(content);
        assert!(
            errors.is_empty(),
            "Expected no errors but got: {:?}",
            errors
        );
    }

    #[test]
    fn test_validate_codeowners_syntax_no_owner() {
        let content = "/src/";

        let errors = validate_codeowners_syntax(content);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].1.contains("No owners specified"));
    }

    #[test]
    fn test_validate_codeowners_syntax_invalid_owner() {
        let content = "* invalid-owner";

        let errors = validate_codeowners_syntax(content);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].1.contains("Invalid owner format"));
    }

    #[test]
    fn test_check_codeowners_syntax_with_errors() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("CODEOWNERS"), "/src/\n* bad-owner").unwrap();

        let scanner = Scanner::new(temp_dir.path().to_path_buf());

        let findings = check_codeowners_syntax(&scanner);
        assert_eq!(findings.len(), 2);
        assert!(findings.iter().all(|f| f.rule_id == "CODE002"));
    }

    // ===== Full Integration Test =====

    #[tokio::test]
    async fn test_codeowners_rules_run() {
        let temp_dir = TempDir::new().unwrap();
        let scanner = Scanner::new(temp_dir.path().to_path_buf());
        let config = Config::default();

        let rules = CodeownersRules;
        let findings = rules.run(&scanner, &config).await.unwrap();

        // Should at least have CODE001 (missing CODEOWNERS)
        assert!(findings.iter().any(|f| f.rule_id == "CODE001"));
    }

    #[tokio::test]
    async fn test_codeowners_rules_with_valid_file() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("CODEOWNERS"), "* @valid-owner").unwrap();

        let scanner = Scanner::new(temp_dir.path().to_path_buf());
        let config = Config::default();

        let rules = CodeownersRules;
        let findings = rules.run(&scanner, &config).await.unwrap();

        // Should not have CODE001 (CODEOWNERS exists)
        assert!(!findings.iter().any(|f| f.rule_id == "CODE001"));
        // Should not have CODE002 (valid syntax)
        assert!(!findings.iter().any(|f| f.rule_id == "CODE002"));
    }

    #[test]
    fn test_find_codeowners_priority() {
        let temp_dir = TempDir::new().unwrap();

        // Create both root and .github CODEOWNERS
        fs::write(temp_dir.path().join("CODEOWNERS"), "root content").unwrap();
        fs::create_dir_all(temp_dir.path().join(".github")).unwrap();
        fs::write(temp_dir.path().join(".github/CODEOWNERS"), "github content").unwrap();

        let scanner = Scanner::new(temp_dir.path().to_path_buf());

        // Root should be preferred
        let (path, content) = find_codeowners(&scanner).unwrap();
        assert_eq!(path, "CODEOWNERS");
        assert_eq!(content, "root content");
    }
}
