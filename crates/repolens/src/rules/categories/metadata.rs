//! Repository metadata rules
//!
//! This module provides rules for checking repository metadata configuration:
//! - Repository description (META001)
//! - Topics/tags (META002)
//! - Website URL (META003)
//! - Social preview image (META004)

use crate::config::Config;
use crate::error::RepoLensError;
use crate::providers::github::GitHubProvider;
use crate::rules::engine::RuleCategory;
use crate::rules::results::{Finding, Severity};
use crate::scanner::Scanner;

use serde::Deserialize;

/// Metadata returned from the GitHub API
#[derive(Debug, Deserialize)]
struct RepoMetadata {
    description: Option<String>,
    #[serde(default)]
    topics: Vec<String>,
    homepage: Option<String>,
    #[serde(rename = "hasPages")]
    #[allow(dead_code)]
    has_pages: Option<bool>,
}

/// Rules for checking repository metadata
pub struct MetadataRules;

#[async_trait::async_trait]
impl RuleCategory for MetadataRules {
    fn name(&self) -> &'static str {
        "metadata"
    }

    async fn run(
        &self,
        _scanner: &Scanner,
        config: &Config,
    ) -> Result<Vec<Finding>, RepoLensError> {
        let mut findings = Vec::new();

        if !GitHubProvider::is_available() {
            return Ok(findings);
        }

        let provider = match GitHubProvider::new() {
            Ok(p) => p,
            Err(_) => return Ok(findings),
        };

        let metadata = match get_repo_metadata(&provider) {
            Ok(m) => m,
            Err(_) => return Ok(findings),
        };

        if config.is_rule_enabled("metadata/description") {
            findings.extend(check_description(&metadata));
        }

        if config.is_rule_enabled("metadata/topics") {
            findings.extend(check_topics(&metadata));
        }

        if config.is_rule_enabled("metadata/homepage") {
            findings.extend(check_homepage(&metadata));
        }

        if config.is_rule_enabled("metadata/social-preview") {
            findings.extend(check_social_preview(&provider));
        }

        Ok(findings)
    }
}

/// Fetch repository metadata from GitHub API
fn get_repo_metadata(provider: &GitHubProvider) -> Result<RepoMetadata, RepoLensError> {
    let output = std::process::Command::new("gh")
        .args([
            "repo",
            "view",
            &format!("{}/{}", provider.owner(), provider.name()),
            "--json",
            "description,topics,homepage,hasPages",
        ])
        .output()
        .map_err(|_| {
            RepoLensError::Provider(crate::error::ProviderError::CommandFailed {
                command: "gh repo view".to_string(),
            })
        })?;

    if !output.status.success() {
        return Err(RepoLensError::Provider(
            crate::error::ProviderError::CommandFailed {
                command: "gh repo view".to_string(),
            },
        ));
    }

    let metadata: RepoMetadata = serde_json::from_slice(&output.stdout)?;
    Ok(metadata)
}

/// META001: Check for repository description
fn check_description(metadata: &RepoMetadata) -> Vec<Finding> {
    let mut findings = Vec::new();

    let has_description = metadata
        .description
        .as_ref()
        .is_some_and(|d| !d.trim().is_empty());

    if !has_description {
        findings.push(
            Finding::new(
                "META001",
                "metadata",
                Severity::Info,
                "Repository description is missing",
            )
            .with_description(
                "A repository description helps users understand the purpose of your project \
                 at a glance and improves discoverability in search results.",
            )
            .with_remediation(
                "Add a description in your repository settings: Settings > General > Description.",
            ),
        );
    }

    findings
}

/// META002: Check for topics/tags
fn check_topics(metadata: &RepoMetadata) -> Vec<Finding> {
    let mut findings = Vec::new();

    if metadata.topics.is_empty() {
        findings.push(
            Finding::new(
                "META002",
                "metadata",
                Severity::Info,
                "No topics or tags configured",
            )
            .with_description(
                "Topics help classify your repository and improve discoverability. \
                 They are used by GitHub's explore and search features.",
            )
            .with_remediation(
                "Add relevant topics to your repository: Settings > General > Topics, \
                 or click 'Add topics' on the repository page.",
            ),
        );
    }

    findings
}

/// META003: Check for website URL
fn check_homepage(metadata: &RepoMetadata) -> Vec<Finding> {
    let mut findings = Vec::new();

    let has_homepage = metadata
        .homepage
        .as_ref()
        .is_some_and(|h| !h.trim().is_empty());

    if !has_homepage {
        findings.push(
            Finding::new(
                "META003",
                "metadata",
                Severity::Info,
                "Website URL is not configured",
            )
            .with_description(
                "A website URL provides users with a link to your project's documentation, \
                 landing page, or related resources.",
            )
            .with_remediation(
                "Add a website URL in your repository settings: Settings > General > Website.",
            ),
        );
    }

    findings
}

/// META004: Check for social preview image
fn check_social_preview(provider: &GitHubProvider) -> Vec<Finding> {
    let mut findings = Vec::new();

    // Check social preview via API - the openGraphImageUrl field
    let output = std::process::Command::new("gh")
        .args([
            "api",
            &format!("repos/{}/{}", provider.owner(), provider.name()),
            "--jq",
            ".has_custom_open_graph_image // false",
        ])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let result = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if result != "true" {
                findings.push(
                    Finding::new(
                        "META004",
                        "metadata",
                        Severity::Info,
                        "Social preview image is missing",
                    )
                    .with_description(
                        "A custom social preview image is displayed when your repository \
                         is shared on social media platforms. It helps make your project \
                         more recognizable and professional.",
                    )
                    .with_remediation(
                        "Upload a social preview image in your repository settings: \
                         Settings > General > Social preview.",
                    ),
                );
            }
        }
        _ => {} // Skip if API call fails
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_description_missing() {
        let metadata = RepoMetadata {
            description: None,
            topics: vec![],
            homepage: None,
            has_pages: None,
        };
        let findings = check_description(&metadata);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "META001");
    }

    #[test]
    fn test_check_description_empty() {
        let metadata = RepoMetadata {
            description: Some("  ".to_string()),
            topics: vec![],
            homepage: None,
            has_pages: None,
        };
        let findings = check_description(&metadata);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "META001");
    }

    #[test]
    fn test_check_description_present() {
        let metadata = RepoMetadata {
            description: Some("A great project".to_string()),
            topics: vec![],
            homepage: None,
            has_pages: None,
        };
        let findings = check_description(&metadata);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_check_topics_missing() {
        let metadata = RepoMetadata {
            description: None,
            topics: vec![],
            homepage: None,
            has_pages: None,
        };
        let findings = check_topics(&metadata);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "META002");
    }

    #[test]
    fn test_check_topics_present() {
        let metadata = RepoMetadata {
            description: None,
            topics: vec!["rust".to_string(), "cli".to_string()],
            homepage: None,
            has_pages: None,
        };
        let findings = check_topics(&metadata);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_check_homepage_missing() {
        let metadata = RepoMetadata {
            description: None,
            topics: vec![],
            homepage: None,
            has_pages: None,
        };
        let findings = check_homepage(&metadata);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "META003");
    }

    #[test]
    fn test_check_homepage_empty() {
        let metadata = RepoMetadata {
            description: None,
            topics: vec![],
            homepage: Some("".to_string()),
            has_pages: None,
        };
        let findings = check_homepage(&metadata);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "META003");
    }

    #[test]
    fn test_check_homepage_present() {
        let metadata = RepoMetadata {
            description: None,
            topics: vec![],
            homepage: Some("https://example.com".to_string()),
            has_pages: None,
        };
        let findings = check_homepage(&metadata);
        assert!(findings.is_empty());
    }
}
