//! GitHub provider - Interactions with GitHub API via gh CLI

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::process::Command;

/// GitHub provider for repository operations
pub struct GitHubProvider {
    repo_owner: String,
    repo_name: String,
}

#[derive(Debug, Deserialize)]
struct RepoInfo {
    name: String,
    owner: RepoOwner,
    visibility: String,
    #[serde(rename = "hasIssuesEnabled")]
    has_issues_enabled: bool,
    #[serde(rename = "hasDiscussionsEnabled")]
    has_discussions_enabled: bool,
    #[serde(rename = "hasWikiEnabled")]
    has_wiki_enabled: bool,
}

#[derive(Debug, Deserialize)]
struct RepoOwner {
    login: String,
}

impl GitHubProvider {
    /// Create a new GitHub provider for the current repository
    pub fn new() -> Result<Self> {
        let (owner, name) = Self::get_repo_info()?;
        Ok(Self {
            repo_owner: owner,
            repo_name: name,
        })
    }

    /// Check if GitHub CLI is available and authenticated
    pub fn is_available() -> bool {
        Command::new("gh")
            .args(["auth", "status"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Get repository owner and name
    fn get_repo_info() -> Result<(String, String)> {
        let output = Command::new("gh")
            .args([
                "repo", "view",
                "--json", "owner,name",
                "-q", ".owner.login + \"/\" + .name"
            ])
            .output()
            .context("Failed to get repository info")?;

        if !output.status.success() {
            bail!("Not in a GitHub repository or not authenticated");
        }

        let full_name = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let parts: Vec<&str> = full_name.split('/').collect();

        if parts.len() != 2 {
            bail!("Invalid repository name format");
        }

        Ok((parts[0].to_string(), parts[1].to_string()))
    }

    /// Get the full repository name (owner/name)
    pub fn full_name(&self) -> String {
        format!("{}/{}", self.repo_owner, self.repo_name)
    }

    /// Get repository visibility
    pub fn get_visibility(&self) -> Result<String> {
        let output = Command::new("gh")
            .args([
                "repo", "view",
                "--json", "visibility",
                "-q", ".visibility"
            ])
            .output()
            .context("Failed to get repository visibility")?;

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_lowercase())
    }

    /// Check if the repository is public
    pub fn is_public(&self) -> Result<bool> {
        Ok(self.get_visibility()? == "public")
    }

    /// Get list of repository secrets (names only)
    pub fn list_secrets(&self) -> Result<Vec<String>> {
        let output = Command::new("gh")
            .args(["secret", "list", "--json", "name", "-q", ".[].name"])
            .output()
            .context("Failed to list secrets")?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        Ok(output_str.lines().map(|s| s.to_string()).collect())
    }

    /// Get list of repository variables
    pub fn list_variables(&self) -> Result<Vec<String>> {
        let output = Command::new("gh")
            .args(["variable", "list", "--json", "name", "-q", ".[].name"])
            .output()
            .context("Failed to list variables")?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        Ok(output_str.lines().map(|s| s.to_string()).collect())
    }

    /// Get branch protection status
    pub fn get_branch_protection(&self, branch: &str) -> Result<Option<BranchProtection>> {
        let output = Command::new("gh")
            .args([
                "api",
                &format!("repos/{}/branches/{}/protection", self.full_name(), branch),
            ])
            .output()
            .context("Failed to get branch protection")?;

        if !output.status.success() {
            // 404 means no protection
            return Ok(None);
        }

        let protection: BranchProtection = serde_json::from_slice(&output.stdout)
            .context("Failed to parse branch protection")?;

        Ok(Some(protection))
    }
}

/// Branch protection settings from GitHub API
#[derive(Debug, Deserialize)]
pub struct BranchProtection {
    #[serde(rename = "required_status_checks")]
    pub required_status_checks: Option<StatusChecks>,

    #[serde(rename = "enforce_admins")]
    pub enforce_admins: Option<EnforceAdmins>,

    #[serde(rename = "required_pull_request_reviews")]
    pub required_pull_request_reviews: Option<PullRequestReviews>,

    #[serde(rename = "required_linear_history")]
    pub required_linear_history: Option<RequiredLinearHistory>,

    #[serde(rename = "allow_force_pushes")]
    pub allow_force_pushes: Option<AllowForcePushes>,

    #[serde(rename = "allow_deletions")]
    pub allow_deletions: Option<AllowDeletions>,
}

#[derive(Debug, Deserialize)]
pub struct StatusChecks {
    pub strict: bool,
    pub contexts: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct EnforceAdmins {
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct PullRequestReviews {
    #[serde(rename = "required_approving_review_count")]
    pub required_approving_review_count: u32,
}

#[derive(Debug, Deserialize)]
pub struct RequiredLinearHistory {
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct AllowForcePushes {
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct AllowDeletions {
    pub enabled: bool,
}
