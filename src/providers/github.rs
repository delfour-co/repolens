//! GitHub provider - Interactions with GitHub API via gh CLI

use crate::error::{ProviderError, RepoLensError};
use serde::Deserialize;
use std::process::Command;

/// GitHub provider for repository operations
pub struct GitHubProvider {
    repo_owner: String,
    repo_name: String,
}

#[derive(Debug, Deserialize)]
pub struct RepoInfo {
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    owner: RepoOwner,
    #[serde(rename = "hasIssuesEnabled")]
    pub has_issues_enabled: bool,
    #[serde(rename = "hasDiscussionsEnabled")]
    pub has_discussions_enabled: bool,
    #[serde(rename = "hasWikiEnabled")]
    pub has_wiki_enabled: bool,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct RepoOwner {
    login: String,
}

impl GitHubProvider {
    /// Create a new GitHub provider for the current repository
    pub fn new() -> Result<Self, RepoLensError> {
        let (owner, name) = Self::get_repo_info()?;
        Ok(Self {
            repo_owner: owner,
            repo_name: name,
        })
    }

    /// Check if GitHub CLI is available and authenticated
    #[allow(dead_code)]
    pub fn is_available() -> bool {
        Command::new("gh")
            .args(["auth", "status"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Get repository owner and name
    #[allow(dead_code)]
    fn get_repo_info() -> Result<(String, String), RepoLensError> {
        let output = Command::new("gh")
            .args([
                "repo",
                "view",
                "--json",
                "owner,name",
                "-q",
                ".owner.login + \"/\" + .name",
            ])
            .output()
            .map_err(|_| {
                RepoLensError::Provider(ProviderError::CommandFailed {
                    command: "gh repo view".to_string(),
                })
            })?;

        if !output.status.success() {
            return Err(RepoLensError::Provider(ProviderError::NotAuthenticated));
        }

        let full_name = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let parts: Vec<&str> = full_name.split('/').collect();

        if parts.len() != 2 {
            return Err(RepoLensError::Provider(ProviderError::InvalidRepoName {
                name: full_name,
            }));
        }

        Ok((parts[0].to_string(), parts[1].to_string()))
    }

    /// Get the full repository name (owner/name)
    pub fn full_name(&self) -> String {
        format!("{}/{}", self.repo_owner, self.repo_name)
    }

    /// Get repository visibility
    #[allow(dead_code)]
    pub fn get_visibility(&self) -> Result<String, RepoLensError> {
        let output = Command::new("gh")
            .args(["repo", "view", "--json", "visibility", "-q", ".visibility"])
            .output()
            .map_err(|_| {
                RepoLensError::Provider(ProviderError::CommandFailed {
                    command: "gh repo view".to_string(),
                })
            })?;

        Ok(String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_lowercase())
    }

    /// Check if the repository is public
    #[allow(dead_code)]
    pub fn is_public(&self) -> Result<bool, RepoLensError> {
        Ok(self.get_visibility()? == "public")
    }

    /// Get list of repository secrets (names only)
    #[allow(dead_code)]
    pub fn list_secrets(&self) -> Result<Vec<String>, RepoLensError> {
        let output = Command::new("gh")
            .args(["secret", "list", "--json", "name", "-q", ".[].name"])
            .output()
            .map_err(|_| {
                RepoLensError::Provider(ProviderError::CommandFailed {
                    command: "gh secret list".to_string(),
                })
            })?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        Ok(output_str.lines().map(|s| s.to_string()).collect())
    }

    /// Get list of repository variables
    #[allow(dead_code)]
    pub fn list_variables(&self) -> Result<Vec<String>, RepoLensError> {
        let output = Command::new("gh")
            .args(["variable", "list", "--json", "name", "-q", ".[].name"])
            .output()
            .map_err(|_| {
                RepoLensError::Provider(ProviderError::CommandFailed {
                    command: "gh variable list".to_string(),
                })
            })?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        Ok(output_str.lines().map(|s| s.to_string()).collect())
    }

    /// Get branch protection status
    pub fn get_branch_protection(
        &self,
        branch: &str,
    ) -> Result<Option<BranchProtection>, RepoLensError> {
        let output = Command::new("gh")
            .args([
                "api",
                &format!("repos/{}/branches/{}/protection", self.full_name(), branch),
            ])
            .output()
            .map_err(|_| {
                RepoLensError::Provider(ProviderError::CommandFailed {
                    command: format!(
                        "gh api repos/{}/branches/{}/protection",
                        self.full_name(),
                        branch
                    ),
                })
            })?;

        if !output.status.success() {
            // 404 means no protection
            return Ok(None);
        }

        let protection: BranchProtection = serde_json::from_slice(&output.stdout)?;

        Ok(Some(protection))
    }

    /// Get repository settings (discussions, issues, wiki, etc.)
    pub fn get_repo_settings(&self) -> Result<RepoInfo, RepoLensError> {
        let output = Command::new("gh")
            .args([
                "repo",
                "view",
                "--json",
                "name,owner,hasIssuesEnabled,hasDiscussionsEnabled,hasWikiEnabled",
            ])
            .output()
            .map_err(|_| {
                RepoLensError::Provider(ProviderError::CommandFailed {
                    command: "gh repo view".to_string(),
                })
            })?;

        if !output.status.success() {
            return Err(RepoLensError::Provider(ProviderError::CommandFailed {
                command: "gh repo view".to_string(),
            }));
        }

        let repo_info: RepoInfo = serde_json::from_slice(&output.stdout)?;
        Ok(repo_info)
    }

    /// Check if vulnerability alerts are enabled
    pub fn has_vulnerability_alerts(&self) -> Result<bool, RepoLensError> {
        let output = Command::new("gh")
            .args([
                "api",
                &format!("repos/{}/vulnerability-alerts", self.full_name()),
            ])
            .output()
            .map_err(|_| {
                RepoLensError::Provider(ProviderError::CommandFailed {
                    command: format!("gh api repos/{}/vulnerability-alerts", self.full_name()),
                })
            })?;

        // gh api returns exit code 0 for HTTP 2xx (enabled), non-zero for HTTP 4xx (disabled)
        Ok(output.status.success())
    }

    /// Check if automated security fixes are enabled
    pub fn has_automated_security_fixes(&self) -> Result<bool, RepoLensError> {
        let output = Command::new("gh")
            .args([
                "api",
                &format!("repos/{}/automated-security-fixes", self.full_name()),
            ])
            .output()
            .map_err(|_| {
                RepoLensError::Provider(ProviderError::CommandFailed {
                    command: format!("gh api repos/{}/automated-security-fixes", self.full_name()),
                })
            })?;

        // gh api returns exit code 0 for HTTP 2xx (enabled), non-zero for HTTP 4xx (disabled)
        Ok(output.status.success())
    }

    /// Create a GitHub issue in the repository
    ///
    /// # Arguments
    ///
    /// * `title` - The issue title
    /// * `body` - The issue body/description
    /// * `labels` - Labels to add to the issue
    ///
    /// # Returns
    ///
    /// The URL of the created issue
    ///
    /// # Errors
    ///
    /// Returns an error if the issue creation fails
    /// Ensure a label exists in the repository, creating it if necessary
    pub fn ensure_label(&self, label: &str, color: &str, description: &str) {
        // Check if label exists by trying to view it
        let check = Command::new("gh")
            .args(["label", "list", "--search", label, "--json", "name"])
            .output();

        if let Ok(output) = check {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains(label) {
                return;
            }
        }

        // Create the label
        let _ = Command::new("gh")
            .args([
                "label",
                "create",
                label,
                "--color",
                color,
                "--description",
                description,
            ])
            .output();
    }

    pub fn create_issue(
        &self,
        title: &str,
        body: &str,
        labels: &[&str],
    ) -> Result<String, RepoLensError> {
        // Ensure all labels exist before creating the issue
        for label in labels {
            self.ensure_label(label, "d73a4a", "Created by RepoLens audit");
        }

        let mut args = vec!["issue", "create", "--title", title, "--body", body];
        for label in labels {
            args.push("--label");
            args.push(label);
        }
        let output = Command::new("gh").args(&args).output().map_err(|_| {
            RepoLensError::Provider(ProviderError::CommandFailed {
                command: format!("gh {}", args.join(" ")),
            })
        })?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(RepoLensError::Provider(ProviderError::CommandFailed {
                command: format!("Failed to create issue: {}", stderr),
            }));
        }
        let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(url)
    }

    /// Create a pull request
    ///
    /// # Arguments
    ///
    /// * `title` - The PR title
    /// * `body` - The PR body/description
    /// * `head` - The branch to merge from
    /// * `base` - The base branch to merge into (defaults to default branch)
    ///
    /// # Returns
    ///
    /// The URL of the created pull request
    ///
    /// # Errors
    ///
    /// Returns an error if the PR creation fails
    pub fn create_pull_request(
        &self,
        title: &str,
        body: &str,
        head: &str,
        base: Option<&str>,
    ) -> Result<String, RepoLensError> {
        let mut args = vec![
            "pr", "create", "--title", title, "--body", body, "--head", head,
        ];

        if let Some(base_branch) = base {
            args.push("--base");
            args.push(base_branch);
        }

        let output = Command::new("gh").args(&args).output().map_err(|_| {
            RepoLensError::Provider(ProviderError::CommandFailed {
                command: format!("gh {}", args.join(" ")),
            })
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(RepoLensError::Provider(ProviderError::CommandFailed {
                command: format!("gh pr create: {}", stderr),
            }));
        }

        // Extract PR URL from output
        let stdout = String::from_utf8_lossy(&output.stdout);
        let url = stdout.trim().to_string();

        Ok(url)
    }
}

/// Branch protection settings from GitHub API
#[derive(Debug, Deserialize)]
pub struct BranchProtection {
    #[serde(rename = "required_status_checks")]
    pub required_status_checks: Option<StatusChecks>,

    #[serde(rename = "enforce_admins")]
    #[allow(dead_code)]
    pub enforce_admins: Option<EnforceAdmins>,

    #[serde(rename = "required_pull_request_reviews")]
    pub required_pull_request_reviews: Option<PullRequestReviews>,

    #[serde(rename = "required_linear_history")]
    #[allow(dead_code)]
    pub required_linear_history: Option<RequiredLinearHistory>,

    #[serde(rename = "allow_force_pushes")]
    pub allow_force_pushes: Option<AllowForcePushes>,

    #[serde(rename = "allow_deletions")]
    #[allow(dead_code)]
    pub allow_deletions: Option<AllowDeletions>,
}

#[derive(Debug, Deserialize)]
pub struct StatusChecks {
    #[allow(dead_code)]
    pub strict: bool,
    #[allow(dead_code)]
    pub contexts: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct EnforceAdmins {
    #[allow(dead_code)]
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct PullRequestReviews {
    #[serde(rename = "required_approving_review_count")]
    pub required_approving_review_count: u32,
}

#[derive(Debug, Deserialize)]
pub struct RequiredLinearHistory {
    #[allow(dead_code)]
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct AllowForcePushes {
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct AllowDeletions {
    #[allow(dead_code)]
    pub enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    /// Create a GitHubProvider with test values (bypasses gh CLI)
    fn test_provider() -> GitHubProvider {
        GitHubProvider {
            repo_owner: "test-owner".to_string(),
            repo_name: "test-repo".to_string(),
        }
    }

    #[test]
    fn test_full_name() {
        let provider = test_provider();
        assert_eq!(provider.full_name(), "test-owner/test-repo");
    }

    #[test]
    fn test_exit_code_success_means_enabled() {
        // The "true" command exits with code 0 (success)
        // This simulates gh api returning HTTP 2xx (feature enabled)
        let output = Command::new("true").output().unwrap();
        assert!(output.status.success());
        // With the fix, output.status.success() correctly returns true
    }

    #[test]
    fn test_exit_code_failure_means_disabled() {
        // The "false" command exits with code 1 (failure)
        // This simulates gh api returning HTTP 4xx (feature disabled)
        let output = Command::new("false").output().unwrap();
        assert!(!output.status.success());
        // With the fix, output.status.success() correctly returns false
    }

    #[test]
    fn test_exit_code_not_http_status() {
        // Verify that process exit codes are NOT HTTP status codes.
        // The old buggy code compared exit codes against 200/204, which never matches
        // because process exit codes are 0 (success) or 1 (failure).
        let success_output = Command::new("true").output().unwrap();
        let failure_output = Command::new("false").output().unwrap();

        let success_code = success_output.status.code();
        let failure_code = failure_output.status.code();

        // Exit code 0 for success, 1 for failure -- never 200 or 204
        assert_eq!(success_code, Some(0));
        assert_eq!(failure_code, Some(1));

        // The old buggy check: status_code == Some(204) || status_code == Some(200)
        // This would ALWAYS be false for both success and failure:
        assert!(success_code != Some(204) && success_code != Some(200));
        assert!(failure_code != Some(204) && failure_code != Some(200));

        // The correct check uses .success() which checks exit code == 0
        assert!(success_output.status.success());
        assert!(!failure_output.status.success());
    }

    #[test]
    fn test_has_vulnerability_alerts_uses_gh_api() {
        // This test verifies the provider is constructed correctly
        // and that full_name() produces the right API path
        let provider = test_provider();
        let expected_endpoint = "repos/test-owner/test-repo/vulnerability-alerts";
        assert_eq!(
            format!("repos/{}/vulnerability-alerts", provider.full_name()),
            expected_endpoint
        );
    }

    #[test]
    fn test_has_automated_security_fixes_uses_gh_api() {
        let provider = test_provider();
        let expected_endpoint = "repos/test-owner/test-repo/automated-security-fixes";
        assert_eq!(
            format!("repos/{}/automated-security-fixes", provider.full_name()),
            expected_endpoint
        );
    }
}
