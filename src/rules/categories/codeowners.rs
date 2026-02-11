//! CODEOWNERS and Releases rules
//!
//! This module provides rules for checking:
//! - CODEOWNERS file presence and validity
//! - GitHub releases and tags
//!
//! ## Rules
//!
//! ### CODEOWNERS Rules
//! - CODE001 (info): CODEOWNERS file is missing
//! - CODE002 (warning): CODEOWNERS file has syntax errors
//! - CODE003 (warning): CODEOWNERS references non-existent users/teams
//!
//! ### Release Rules
//! - REL001 (info): No releases published
//! - REL002 (warning): Last release is older than 1 year
//! - REL003 (info): Tags are not signed

use crate::config::Config;
use crate::error::RepoLensError;
use crate::rules::engine::RuleCategory;
use crate::rules::results::{Finding, Severity};
use crate::scanner::Scanner;
use chrono::{DateTime, Utc};
use regex::Regex;
use serde::Deserialize;
use std::process::Command;

/// Rules for checking CODEOWNERS and releases
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

        if config.is_rule_enabled("codeowners/valid-owners") {
            findings.extend(check_codeowners_valid_owners(scanner).await);
        }

        // Release rules
        if config.is_rule_enabled("codeowners/releases") {
            findings.extend(check_releases().await);
        }

        if config.is_rule_enabled("codeowners/signed-tags") {
            findings.extend(check_signed_tags(scanner));
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

/// CODE003: Check if CODEOWNERS references valid users/teams
async fn check_codeowners_valid_owners(scanner: &Scanner) -> Vec<Finding> {
    let mut findings = Vec::new();

    let Some((path, content)) = find_codeowners(scanner) else {
        return findings;
    };

    // Extract all owner references
    let owners = extract_owners(&content);

    // Try to validate via GitHub API
    let invalid_owners = validate_owners_via_github(&owners).await;

    for (owner, line_num) in invalid_owners {
        findings.push(
            Finding::new(
                "CODE003",
                "codeowners",
                Severity::Warning,
                format!("CODEOWNERS references potentially invalid owner: {}", owner),
            )
            .with_location(format!("{}:{}", path, line_num))
            .with_description(
                "The referenced user or team may not exist or may not have access to \
                 this repository. GitHub will not assign them as reviewers.",
            )
            .with_remediation(
                "Verify that the user/team exists and has access to this repository:\n\
                 - For users: Check the username is correct\n\
                 - For teams: Use the format @org/team-name\n\
                 - Ensure the user/team has at least read access to the repository",
            ),
        );
    }

    findings
}

/// Extract all owner references with line numbers from CODEOWNERS content
fn extract_owners(content: &str) -> Vec<(String, usize)> {
    let mut owners = Vec::new();
    let owner_pattern = Regex::new(r"@[\w\-\.]+(/[\w\-\.]+)?").unwrap();

    for (line_num, line) in content.lines().enumerate() {
        let line_num = line_num + 1;
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        for cap in owner_pattern.find_iter(line) {
            owners.push((cap.as_str().to_string(), line_num));
        }
    }

    owners
}

/// Validate owners via GitHub API (returns invalid ones)
async fn validate_owners_via_github(owners: &[(String, usize)]) -> Vec<(String, usize)> {
    let mut invalid = Vec::new();

    for (owner, line_num) in owners {
        let owner_name = owner.trim_start_matches('@');

        // Check if it's a team reference (contains /)
        let is_valid = if owner_name.contains('/') {
            // Team reference: @org/team
            check_team_exists(owner_name)
        } else {
            // User reference: @username
            check_user_exists(owner_name)
        };

        if !is_valid {
            invalid.push((owner.clone(), *line_num));
        }
    }

    invalid
}

/// Check if a GitHub user exists using gh CLI
fn check_user_exists(username: &str) -> bool {
    let output = Command::new("gh")
        .args(["api", &format!("users/{}", username)])
        .output();

    match output {
        Ok(o) => o.status.success(),
        Err(_) => true, // If gh is not available, assume valid
    }
}

/// Check if a GitHub team exists using gh CLI
fn check_team_exists(team_ref: &str) -> bool {
    let parts: Vec<&str> = team_ref.split('/').collect();
    if parts.len() != 2 {
        return false;
    }

    let org = parts[0];
    let team = parts[1];

    let output = Command::new("gh")
        .args(["api", &format!("orgs/{}/teams/{}", org, team)])
        .output();

    match output {
        Ok(o) => o.status.success(),
        Err(_) => true, // If gh is not available, assume valid
    }
}

/// GitHub Release structure
#[derive(Debug, Deserialize)]
struct Release {
    #[serde(rename = "tagName")]
    tag_name: String,
    #[serde(rename = "publishedAt")]
    published_at: String,
    #[serde(rename = "isDraft")]
    is_draft: bool,
}

/// REL001 & REL002: Check releases
async fn check_releases() -> Vec<Finding> {
    let mut findings = Vec::new();

    // Get releases via gh CLI
    let output = Command::new("gh")
        .args([
            "release",
            "list",
            "--json",
            "tagName,publishedAt,isDraft",
            "--limit",
            "10",
        ])
        .output();

    let releases: Vec<Release> = match output {
        Ok(o) if o.status.success() => serde_json::from_slice(&o.stdout).unwrap_or_default(),
        _ => {
            // gh CLI not available or failed - skip these checks
            return findings;
        }
    };

    // Filter out drafts
    let published_releases: Vec<&Release> = releases.iter().filter(|r| !r.is_draft).collect();

    // REL001: No releases
    if published_releases.is_empty() {
        findings.push(
            Finding::new(
                "REL001",
                "codeowners",
                Severity::Info,
                "No releases have been published",
            )
            .with_description(
                "GitHub releases help users track versions and changes. Publishing releases \
                 makes it easier for users to download specific versions and see changelogs.",
            )
            .with_remediation(
                "Create a release using GitHub's release feature or the gh CLI:\n\
                 gh release create v1.0.0 --title \"v1.0.0\" --notes \"Initial release\"\n\n\
                 Consider using semantic versioning (MAJOR.MINOR.PATCH).",
            ),
        );
        return findings;
    }

    // REL002: Check if last release is older than 1 year
    if let Some(latest) = published_releases.first() {
        if let Ok(published) = DateTime::parse_from_rfc3339(&latest.published_at) {
            let now = Utc::now();
            let age = now.signed_duration_since(published.with_timezone(&Utc));

            if age.num_days() > 365 {
                let years = age.num_days() / 365;
                findings.push(
                    Finding::new(
                        "REL002",
                        "codeowners",
                        Severity::Warning,
                        format!(
                            "Last release ({}) is over {} year(s) old",
                            latest.tag_name, years
                        ),
                    )
                    .with_description(
                        "Having outdated releases may indicate the project is unmaintained \
                         or that users are not getting the latest features and fixes.",
                    )
                    .with_remediation(
                        "Consider creating a new release with the latest changes:\n\
                         gh release create vX.Y.Z --generate-notes\n\n\
                         If the project is actively maintained, regular releases help users \
                         track progress and adopt new features.",
                    ),
                );
            }
        }
    }

    findings
}

/// REL003: Check for unsigned tags
fn check_signed_tags(scanner: &Scanner) -> Vec<Finding> {
    let mut findings = Vec::new();

    // Get tags and check if they're signed
    let output = Command::new("git")
        .args(["tag", "-l", "--format=%(refname:short) %(objecttype)"])
        .current_dir(scanner.root())
        .output();

    let tags_output = match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        _ => return findings,
    };

    let tags: Vec<&str> = tags_output.lines().collect();

    if tags.is_empty() {
        return findings;
    }

    // Check for signed tags
    let mut unsigned_tags = Vec::new();

    for line in &tags {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let tag_name = parts[0];
            let object_type = parts[1];

            // Annotated tags have type "tag", lightweight have "commit"
            // For annotated tags, check signature
            if object_type == "tag" {
                let verify = Command::new("git")
                    .args(["tag", "-v", tag_name])
                    .current_dir(scanner.root())
                    .output();

                match verify {
                    Ok(o) if !o.status.success() => {
                        unsigned_tags.push(tag_name.to_string());
                    }
                    _ => {}
                }
            } else {
                // Lightweight tags are never signed
                unsigned_tags.push(tag_name.to_string());
            }
        }
    }

    if !unsigned_tags.is_empty() {
        let tag_list = if unsigned_tags.len() > 5 {
            format!(
                "{} and {} more",
                unsigned_tags[..5].join(", "),
                unsigned_tags.len() - 5
            )
        } else {
            unsigned_tags.join(", ")
        };

        findings.push(
            Finding::new(
                "REL003",
                "codeowners",
                Severity::Info,
                format!("Unsigned tags found: {}", tag_list),
            )
            .with_description(
                "Signed tags provide cryptographic proof of authenticity, helping users \
                 verify that releases came from a trusted source.",
            )
            .with_remediation(
                "Create signed tags using GPG:\n\
                 1. Set up GPG signing: git config --global user.signingkey YOUR_KEY_ID\n\
                 2. Create signed tag: git tag -s v1.0.0 -m \"Version 1.0.0\"\n\n\
                 For existing tags, you can recreate them as signed tags.",
            ),
        );
    }

    findings
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

    // ===== Extract Owners Tests =====

    #[test]
    fn test_extract_owners() {
        let content = r#"
# Comment
* @global-owner
/src/ @team1 @team2
/docs/ @org/docs-team
"#;

        let owners = extract_owners(content);
        assert_eq!(owners.len(), 4);
        assert!(owners.iter().any(|(o, _)| o == "@global-owner"));
        assert!(owners.iter().any(|(o, _)| o == "@team1"));
        assert!(owners.iter().any(|(o, _)| o == "@team2"));
        assert!(owners.iter().any(|(o, _)| o == "@org/docs-team"));
    }

    #[test]
    fn test_extract_owners_with_line_numbers() {
        let content = "* @owner1\n/src/ @owner2";

        let owners = extract_owners(content);
        assert_eq!(owners.len(), 2);
        assert_eq!(owners[0], ("@owner1".to_string(), 1));
        assert_eq!(owners[1], ("@owner2".to_string(), 2));
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
