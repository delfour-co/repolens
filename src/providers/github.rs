//! GitHub provider - Interactions with GitHub API via octocrab (with gh CLI fallback)
//!
//! This module provides GitHub API access using octocrab for direct API calls,
//! with automatic fallback to gh CLI when octocrab is unavailable.
//!
//! ## Authentication
//!
//! The provider supports two authentication methods (in order of preference):
//! 1. `GITHUB_TOKEN` environment variable - Used by octocrab
//! 2. `gh auth login` - Fallback via gh CLI

use crate::error::{ProviderError, RepoLensError};
use crate::providers::{RepoMetadata, RepoProvider};
use octocrab::Octocrab;
use serde::Deserialize;
use std::env;
use std::future::Future;
use std::process::Command;
use tokio::runtime::Runtime;

/// GitHub provider for repository operations
///
/// Provides access to GitHub API for:
/// - Repository information
/// - Branch protection settings
/// - Security features (vulnerability alerts, automated fixes)
/// - Issue and PR creation
pub struct GitHubProvider {
    repo_owner: String,
    repo_name: String,
    octocrab: Option<Octocrab>,
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

impl RepoInfo {
    /// Build a [`RepoInfo`] from GitLab project settings.
    ///
    /// GitLab has no "discussions" concept, so `has_discussions_enabled` is
    /// always `false`; the kept checks only read issues / wiki here.
    pub fn from_gitlab(
        name: &str,
        owner: &str,
        has_issues_enabled: bool,
        has_wiki_enabled: bool,
    ) -> Self {
        Self {
            name: name.to_string(),
            owner: RepoOwner {
                login: owner.to_string(),
            },
            has_issues_enabled,
            has_discussions_enabled: false,
            has_wiki_enabled,
        }
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct RepoOwner {
    login: String,
}

impl GitHubProvider {
    /// Create a new GitHub provider for the current repository
    ///
    /// Attempts to authenticate via GITHUB_TOKEN first, then falls back to gh CLI.
    pub fn new() -> Result<Self, RepoLensError> {
        let (owner, name) = Self::get_repo_info()?;

        // Try to create octocrab instance with GITHUB_TOKEN
        let octocrab = Self::create_octocrab_client();

        Ok(Self {
            repo_owner: owner,
            repo_name: name,
            octocrab,
        })
    }

    /// Create an octocrab client if GITHUB_TOKEN is available
    fn create_octocrab_client() -> Option<Octocrab> {
        env::var("GITHUB_TOKEN")
            .ok()
            .and_then(|token| Octocrab::builder().personal_token(token).build().ok())
    }

    /// Check if GitHub API is available (via token or gh CLI)
    pub fn is_available() -> bool {
        // Check GITHUB_TOKEN first
        if env::var("GITHUB_TOKEN").is_ok() {
            return true;
        }

        // Fall back to gh CLI check
        Command::new("gh")
            .args(["auth", "status"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Check if GITHUB_TOKEN is set
    #[allow(dead_code)]
    pub fn has_token() -> bool {
        env::var("GITHUB_TOKEN").is_ok()
    }

    /// Get repository owner and name from git remote
    fn get_repo_info() -> Result<(String, String), RepoLensError> {
        // First try parsing git remote URL directly
        if let Ok((owner, name)) = Self::get_repo_from_git_remote() {
            return Ok((owner, name));
        }

        // Fall back to gh CLI
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
        Self::parse_repo_name(&full_name)
    }

    /// Parse owner/name from git remote URL
    fn get_repo_from_git_remote() -> Result<(String, String), RepoLensError> {
        let output = Command::new("git")
            .args(["remote", "get-url", "origin"])
            .output()
            .map_err(|_| {
                RepoLensError::Provider(ProviderError::CommandFailed {
                    command: "git remote get-url origin".to_string(),
                })
            })?;

        if !output.status.success() {
            return Err(RepoLensError::Provider(ProviderError::CommandFailed {
                command: "git remote get-url origin".to_string(),
            }));
        }

        let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Self::parse_github_url(&url)
    }

    /// Parse GitHub URL to extract owner and repo name
    fn parse_github_url(url: &str) -> Result<(String, String), RepoLensError> {
        // Handle SSH URLs: git@github.com:owner/repo.git
        if url.starts_with("git@github.com:") {
            let path = url.trim_start_matches("git@github.com:");
            let path = path.trim_end_matches(".git");
            return Self::parse_repo_name(path);
        }

        // Handle HTTPS URLs: https://github.com/owner/repo.git
        if url.contains("github.com") {
            let path = url
                .split("github.com/")
                .nth(1)
                .ok_or_else(|| {
                    RepoLensError::Provider(ProviderError::InvalidRepoName {
                        name: url.to_string(),
                    })
                })?
                .trim_end_matches(".git");
            return Self::parse_repo_name(path);
        }

        Err(RepoLensError::Provider(ProviderError::InvalidRepoName {
            name: url.to_string(),
        }))
    }

    /// Parse "owner/name" format
    fn parse_repo_name(full_name: &str) -> Result<(String, String), RepoLensError> {
        let parts: Vec<&str> = full_name.split('/').collect();
        if parts.len() != 2 {
            return Err(RepoLensError::Provider(ProviderError::InvalidRepoName {
                name: full_name.to_string(),
            }));
        }
        Ok((parts[0].to_string(), parts[1].to_string()))
    }

    /// Get the repository owner
    #[allow(dead_code)]
    pub fn owner(&self) -> &str {
        &self.repo_owner
    }

    /// Get the repository name
    #[allow(dead_code)]
    pub fn name(&self) -> &str {
        &self.repo_name
    }

    /// Get the full repository name (owner/name)
    pub fn full_name(&self) -> String {
        format!("{}/{}", self.repo_owner, self.repo_name)
    }

    /// Get a reference to the octocrab instance (if available)
    #[allow(dead_code)]
    pub fn octocrab(&self) -> Option<&Octocrab> {
        self.octocrab.as_ref()
    }

    /// Run an async future in a blocking fashion
    /// This is used to bridge async octocrab calls with synchronous code
    fn block_on<F: Future>(future: F) -> F::Output {
        Runtime::new()
            .expect("Failed to create tokio runtime")
            .block_on(future)
    }

    /// Get branch protection status
    pub fn get_branch_protection(
        &self,
        branch: &str,
    ) -> Result<Option<BranchProtection>, RepoLensError> {
        // Use gh CLI for branch protection (octocrab requires more complex setup)
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
    ///
    /// Uses octocrab when GITHUB_TOKEN is available, falls back to gh CLI otherwise.
    pub fn get_repo_settings(&self) -> Result<RepoInfo, RepoLensError> {
        // Try octocrab first if available
        if let Some(octo) = &self.octocrab {
            let owner = self.repo_owner.clone();
            let name = self.repo_name.clone();
            let octo = octo.clone();

            let result = Self::block_on(async move { octo.repos(&owner, &name).get().await });

            if let Ok(repo) = result {
                return Ok(RepoInfo {
                    name: repo.name.clone(),
                    owner: RepoOwner {
                        login: repo
                            .owner
                            .as_ref()
                            .map(|o| o.login.clone())
                            .unwrap_or_default(),
                    },
                    has_issues_enabled: repo.has_issues.unwrap_or(false),
                    // has_discussions not available in octocrab Repository model
                    // This is a limitation - gh CLI provides more complete data
                    has_discussions_enabled: false,
                    has_wiki_enabled: repo.has_wiki.unwrap_or(false),
                });
            }
            // Fall through to gh CLI if octocrab fails
        }

        // Fall back to gh CLI
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

    /// Check if Dependabot security updates are enabled
    pub fn has_dependabot_security_updates(&self) -> Result<bool, RepoLensError> {
        // Dependabot security updates is determined by the automated-security-fixes endpoint
        // which returns enabled/disabled status
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

        if !output.status.success() {
            return Ok(false);
        }

        // Parse the response to check if enabled
        #[derive(Deserialize)]
        struct AutomatedSecurityFixes {
            enabled: bool,
        }

        let response: Result<AutomatedSecurityFixes, _> = serde_json::from_slice(&output.stdout);
        Ok(response.map(|r| r.enabled).unwrap_or(false))
    }

    /// Get secret scanning settings
    pub fn get_secret_scanning(&self) -> Result<SecretScanningSettings, RepoLensError> {
        // Get code security configuration via the repository endpoint
        let output = Command::new("gh")
            .args([
                "api",
                &format!("repos/{}", self.full_name()),
                "--jq",
                ".security_and_analysis",
            ])
            .output()
            .map_err(|_| {
                RepoLensError::Provider(ProviderError::CommandFailed {
                    command: format!("gh api repos/{}", self.full_name()),
                })
            })?;

        if !output.status.success() {
            // If the API call fails, assume secret scanning is disabled
            return Ok(SecretScanningSettings {
                enabled: false,
                push_protection_enabled: false,
            });
        }

        // Parse the security_and_analysis object
        #[derive(Deserialize)]
        struct SecurityStatus {
            status: String,
        }

        #[derive(Deserialize)]
        struct SecurityAndAnalysis {
            secret_scanning: Option<SecurityStatus>,
            secret_scanning_push_protection: Option<SecurityStatus>,
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let security: Result<SecurityAndAnalysis, _> = serde_json::from_str(&stdout);

        match security {
            Ok(s) => Ok(SecretScanningSettings {
                enabled: s
                    .secret_scanning
                    .map(|ss| ss.status == "enabled")
                    .unwrap_or(false),
                push_protection_enabled: s
                    .secret_scanning_push_protection
                    .map(|ss| ss.status == "enabled")
                    .unwrap_or(false),
            }),
            Err(_) => Ok(SecretScanningSettings {
                enabled: false,
                push_protection_enabled: false,
            }),
        }
    }

    /// Get actions permissions for the repository
    pub fn get_actions_permissions(&self) -> Result<ActionsPermissions, RepoLensError> {
        let output = Command::new("gh")
            .args([
                "api",
                &format!("repos/{}/actions/permissions", self.full_name()),
            ])
            .output()
            .map_err(|_| {
                RepoLensError::Provider(ProviderError::CommandFailed {
                    command: format!("gh api repos/{}/actions/permissions", self.full_name()),
                })
            })?;

        if !output.status.success() {
            // If the API call fails, return default (restrictive) permissions
            return Ok(ActionsPermissions {
                enabled: true,
                allowed_actions: Some("all".to_string()),
                default_workflow_permissions: Some("write".to_string()),
                can_approve_pull_request_reviews: Some(true),
            });
        }

        let permissions: ActionsPermissions = serde_json::from_slice(&output.stdout)?;
        Ok(permissions)
    }

    /// Get actions workflow permissions (default permissions for GITHUB_TOKEN)
    pub fn get_actions_workflow_permissions(&self) -> Result<ActionsPermissions, RepoLensError> {
        let output = Command::new("gh")
            .args([
                "api",
                &format!("repos/{}/actions/permissions/workflow", self.full_name()),
            ])
            .output()
            .map_err(|_| {
                RepoLensError::Provider(ProviderError::CommandFailed {
                    command: format!(
                        "gh api repos/{}/actions/permissions/workflow",
                        self.full_name()
                    ),
                })
            })?;

        if !output.status.success() {
            // If the API call fails, assume permissive defaults
            return Ok(ActionsPermissions {
                enabled: true,
                allowed_actions: None,
                default_workflow_permissions: Some("write".to_string()),
                can_approve_pull_request_reviews: Some(true),
            });
        }

        #[derive(Deserialize)]
        struct WorkflowPermissions {
            default_workflow_permissions: Option<String>,
            can_approve_pull_request_reviews: Option<bool>,
        }

        let perms: WorkflowPermissions = serde_json::from_slice(&output.stdout)?;
        Ok(ActionsPermissions {
            enabled: true,
            allowed_actions: None,
            default_workflow_permissions: perms.default_workflow_permissions,
            can_approve_pull_request_reviews: perms.can_approve_pull_request_reviews,
        })
    }

    /// Check if fork pull request workflows require approval
    pub fn get_fork_pr_workflows_policy(&self) -> Result<bool, RepoLensError> {
        // Check repository settings for requiring approval for fork PRs
        // This is available via the actions/permissions/access endpoint
        let output = Command::new("gh")
            .args([
                "api",
                &format!("repos/{}/actions/permissions/access", self.full_name()),
            ])
            .output()
            .map_err(|_| {
                RepoLensError::Provider(ProviderError::CommandFailed {
                    command: format!(
                        "gh api repos/{}/actions/permissions/access",
                        self.full_name()
                    ),
                })
            })?;

        if !output.status.success() {
            // If the API call fails, assume no approval required
            return Ok(false);
        }

        #[derive(Deserialize)]
        struct AccessLevel {
            access_level: Option<String>,
        }

        let access: Result<AccessLevel, _> = serde_json::from_slice(&output.stdout);
        // access_level "none" means only repo collaborators can run, which requires approval
        // "user" or "organization" means more permissive
        Ok(access
            .map(|a| a.access_level.as_deref() == Some("none"))
            .unwrap_or(false))
    }

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

    /// Create a GitHub issue in the repository
    ///
    /// Uses octocrab when GITHUB_TOKEN is available, falls back to gh CLI otherwise.
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

        // Try octocrab first if available
        if let Some(octo) = &self.octocrab {
            let owner = self.repo_owner.clone();
            let name = self.repo_name.clone();
            let octo = octo.clone();
            let title = title.to_string();
            let body = body.to_string();
            let labels: Vec<String> = labels.iter().map(|s| s.to_string()).collect();

            let result = Self::block_on(async move {
                octo.issues(&owner, &name)
                    .create(&title)
                    .body(&body)
                    .labels(labels)
                    .send()
                    .await
            });

            if let Ok(issue) = result {
                return Ok(issue.html_url.to_string());
            }
            // Fall through to gh CLI if octocrab fails
        }

        // Fall back to gh CLI
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

        let stdout = String::from_utf8_lossy(&output.stdout);
        let url = stdout.trim().to_string();
        Ok(url)
    }
}

impl RepoProvider for GitHubProvider {
    fn owner(&self) -> &str {
        &self.repo_owner
    }

    fn name(&self) -> &str {
        &self.repo_name
    }

    /// Fetch repository metadata (description, topics, homepage) via `gh repo view`.
    fn repo_metadata(&self) -> Result<RepoMetadata, RepoLensError> {
        let output = Command::new("gh")
            .args([
                "repo",
                "view",
                &self.full_name(),
                "--json",
                "description,topics,homepage,hasPages",
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

        let metadata: RepoMetadata = serde_json::from_slice(&output.stdout)?;
        Ok(metadata)
    }

    fn get_branch_protection(
        &self,
        branch: &str,
    ) -> Result<Option<BranchProtection>, RepoLensError> {
        GitHubProvider::get_branch_protection(self, branch)
    }

    fn get_repo_settings(&self) -> Result<RepoInfo, RepoLensError> {
        GitHubProvider::get_repo_settings(self)
    }

    fn has_vulnerability_alerts(&self) -> Result<bool, RepoLensError> {
        GitHubProvider::has_vulnerability_alerts(self)
    }

    fn has_automated_security_fixes(&self) -> Result<bool, RepoLensError> {
        GitHubProvider::has_automated_security_fixes(self)
    }

    fn has_dependabot_security_updates(&self) -> Result<bool, RepoLensError> {
        GitHubProvider::has_dependabot_security_updates(self)
    }

    fn get_secret_scanning(&self) -> Result<SecretScanningSettings, RepoLensError> {
        GitHubProvider::get_secret_scanning(self)
    }

    fn get_actions_permissions(&self) -> Result<ActionsPermissions, RepoLensError> {
        GitHubProvider::get_actions_permissions(self)
    }

    fn get_actions_workflow_permissions(&self) -> Result<ActionsPermissions, RepoLensError> {
        GitHubProvider::get_actions_workflow_permissions(self)
    }

    fn get_fork_pr_workflows_policy(&self) -> Result<bool, RepoLensError> {
        GitHubProvider::get_fork_pr_workflows_policy(self)
    }
}

/// Secret scanning settings from GitHub API
#[derive(Debug, Clone, Deserialize)]
pub struct SecretScanningSettings {
    /// Whether secret scanning is enabled
    pub enabled: bool,
    /// Whether push protection is enabled
    pub push_protection_enabled: bool,
}

/// Actions permissions settings from GitHub API
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct ActionsPermissions {
    /// Whether actions are enabled
    pub enabled: bool,
    /// Allowed actions: "all", "local_only", "selected"
    pub allowed_actions: Option<String>,
    /// Default workflow permissions: "read" or "write"
    pub default_workflow_permissions: Option<String>,
    /// Whether actions can approve pull request reviews
    pub can_approve_pull_request_reviews: Option<bool>,
}

/// Fork pull request workflow approval settings
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct ForkPullRequestWorkflowsPolicy {
    /// Approval requirement: "none", "contributor", "everyone"
    pub default_workflow_permissions: Option<String>,
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

impl BranchProtection {
    /// Build a [`BranchProtection`] from GitLab protected-branch state.
    ///
    /// GitLab exposes a force-push toggle and (via approval rules) a required
    /// approval count. The remaining GitHub-shaped fields have no GitLab
    /// equivalent and are left `None`.
    pub fn from_gitlab(allow_force_push: bool, required_approvals: u32) -> Self {
        Self {
            required_status_checks: None,
            enforce_admins: None,
            required_pull_request_reviews: (required_approvals > 0).then_some(PullRequestReviews {
                required_approving_review_count: required_approvals,
            }),
            required_linear_history: None,
            allow_force_pushes: Some(AllowForcePushes {
                enabled: allow_force_push,
            }),
            allow_deletions: None,
        }
    }
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

    /// Create a GitHubProvider with test values (bypasses API calls)
    fn test_provider() -> GitHubProvider {
        GitHubProvider {
            repo_owner: "test-owner".to_string(),
            repo_name: "test-repo".to_string(),
            octocrab: None,
        }
    }

    #[test]
    fn test_full_name() {
        let provider = test_provider();
        assert_eq!(provider.full_name(), "test-owner/test-repo");
    }

    #[test]
    fn test_owner_and_name() {
        let provider = test_provider();
        assert_eq!(provider.owner(), "test-owner");
        assert_eq!(provider.name(), "test-repo");
    }

    #[test]
    fn test_parse_github_url_https() {
        let result = GitHubProvider::parse_github_url("https://github.com/owner/repo.git");
        assert!(result.is_ok());
        let (owner, name) = result.unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(name, "repo");
    }

    #[test]
    fn test_parse_github_url_https_no_git() {
        let result = GitHubProvider::parse_github_url("https://github.com/owner/repo");
        assert!(result.is_ok());
        let (owner, name) = result.unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(name, "repo");
    }

    #[test]
    fn test_parse_github_url_ssh() {
        let result = GitHubProvider::parse_github_url("git@github.com:owner/repo.git");
        assert!(result.is_ok());
        let (owner, name) = result.unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(name, "repo");
    }

    #[test]
    fn test_parse_github_url_invalid() {
        let result = GitHubProvider::parse_github_url("https://gitlab.com/owner/repo.git");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_repo_name_valid() {
        let result = GitHubProvider::parse_repo_name("owner/repo");
        assert!(result.is_ok());
        let (owner, name) = result.unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(name, "repo");
    }

    #[test]
    fn test_parse_repo_name_invalid() {
        let result = GitHubProvider::parse_repo_name("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_has_token_without_env() {
        // Clear the env var if set (in test isolation)
        let original = env::var("GITHUB_TOKEN").ok();
        // TODO: Audit that the environment access only happens in single-threaded code.
        unsafe { env::remove_var("GITHUB_TOKEN") };

        assert!(!GitHubProvider::has_token());

        // Restore original value
        if let Some(val) = original {
            // TODO: Audit that the environment access only happens in single-threaded code.
            unsafe { env::set_var("GITHUB_TOKEN", val) };
        }
    }

    #[test]
    fn test_is_available_returns_bool() {
        // Just verify is_available() doesn't panic and returns a bool
        let _result: bool = GitHubProvider::is_available();
    }

    #[test]
    fn test_octocrab_accessor() {
        let provider = test_provider();
        // Test provider has no octocrab instance
        assert!(provider.octocrab().is_none());
    }

    #[test]
    fn test_branch_protection_deserialization() {
        let json = r#"{
            "required_status_checks": {
                "strict": true,
                "contexts": ["ci/test", "ci/build"]
            },
            "enforce_admins": {
                "enabled": true
            },
            "required_pull_request_reviews": {
                "required_approving_review_count": 2
            },
            "required_linear_history": {
                "enabled": false
            },
            "allow_force_pushes": {
                "enabled": false
            },
            "allow_deletions": {
                "enabled": false
            }
        }"#;

        let protection: BranchProtection = serde_json::from_str(json).unwrap();

        assert!(protection.required_status_checks.is_some());
        let status_checks = protection.required_status_checks.unwrap();
        assert!(status_checks.strict);
        assert_eq!(status_checks.contexts.len(), 2);

        assert!(protection.required_pull_request_reviews.is_some());
        assert_eq!(
            protection
                .required_pull_request_reviews
                .unwrap()
                .required_approving_review_count,
            2
        );

        assert!(protection.allow_force_pushes.is_some());
        assert!(!protection.allow_force_pushes.unwrap().enabled);
    }

    #[test]
    fn test_branch_protection_minimal_deserialization() {
        let json = r#"{}"#;
        let protection: BranchProtection = serde_json::from_str(json).unwrap();

        assert!(protection.required_status_checks.is_none());
        assert!(protection.required_pull_request_reviews.is_none());
        assert!(protection.allow_force_pushes.is_none());
    }

    #[test]
    fn test_repo_info_deserialization() {
        let json = r#"{
            "name": "test-repo",
            "owner": {
                "login": "test-owner"
            },
            "hasIssuesEnabled": true,
            "hasDiscussionsEnabled": false,
            "hasWikiEnabled": true
        }"#;

        let repo_info: RepoInfo = serde_json::from_str(json).unwrap();
        assert!(repo_info.has_issues_enabled);
        assert!(!repo_info.has_discussions_enabled);
        assert!(repo_info.has_wiki_enabled);
    }

    #[test]
    fn test_provider_full_name_format() {
        let provider = GitHubProvider {
            repo_owner: "my-org".to_string(),
            repo_name: "my-repo".to_string(),
            octocrab: None,
        };
        assert_eq!(provider.full_name(), "my-org/my-repo");
    }

    #[test]
    fn test_status_checks_deserialization() {
        let json = r#"{
            "strict": false,
            "contexts": []
        }"#;
        let checks: StatusChecks = serde_json::from_str(json).unwrap();
        assert!(!checks.strict);
        assert!(checks.contexts.is_empty());
    }

    #[test]
    fn test_pull_request_reviews_deserialization() {
        let json = r#"{
            "required_approving_review_count": 1
        }"#;
        let reviews: PullRequestReviews = serde_json::from_str(json).unwrap();
        assert_eq!(reviews.required_approving_review_count, 1);
    }

    #[test]
    fn test_allow_force_pushes_deserialization() {
        let json_enabled = r#"{"enabled": true}"#;
        let json_disabled = r#"{"enabled": false}"#;

        let enabled: AllowForcePushes = serde_json::from_str(json_enabled).unwrap();
        let disabled: AllowForcePushes = serde_json::from_str(json_disabled).unwrap();

        assert!(enabled.enabled);
        assert!(!disabled.enabled);
    }

    #[test]
    fn test_secret_scanning_settings_deserialization() {
        let json = r#"{
            "enabled": true,
            "push_protection_enabled": false
        }"#;

        let settings: SecretScanningSettings = serde_json::from_str(json).unwrap();
        assert!(settings.enabled);
        assert!(!settings.push_protection_enabled);
    }

    #[test]
    fn test_secret_scanning_settings_both_enabled() {
        let json = r#"{
            "enabled": true,
            "push_protection_enabled": true
        }"#;

        let settings: SecretScanningSettings = serde_json::from_str(json).unwrap();
        assert!(settings.enabled);
        assert!(settings.push_protection_enabled);
    }

    #[test]
    fn test_secret_scanning_settings_both_disabled() {
        let json = r#"{
            "enabled": false,
            "push_protection_enabled": false
        }"#;

        let settings: SecretScanningSettings = serde_json::from_str(json).unwrap();
        assert!(!settings.enabled);
        assert!(!settings.push_protection_enabled);
    }

    #[test]
    fn test_actions_permissions_deserialization_full() {
        let json = r#"{
            "enabled": true,
            "allowed_actions": "selected",
            "default_workflow_permissions": "read",
            "can_approve_pull_request_reviews": false
        }"#;

        let perms: ActionsPermissions = serde_json::from_str(json).unwrap();
        assert!(perms.enabled);
        assert_eq!(perms.allowed_actions, Some("selected".to_string()));
        assert_eq!(perms.default_workflow_permissions, Some("read".to_string()));
        assert_eq!(perms.can_approve_pull_request_reviews, Some(false));
    }

    #[test]
    fn test_actions_permissions_deserialization_minimal() {
        let json = r#"{
            "enabled": false
        }"#;

        let perms: ActionsPermissions = serde_json::from_str(json).unwrap();
        assert!(!perms.enabled);
        assert!(perms.allowed_actions.is_none());
        assert!(perms.default_workflow_permissions.is_none());
        assert!(perms.can_approve_pull_request_reviews.is_none());
    }

    #[test]
    fn test_actions_permissions_all_allowed() {
        let json = r#"{
            "enabled": true,
            "allowed_actions": "all",
            "default_workflow_permissions": "write",
            "can_approve_pull_request_reviews": true
        }"#;

        let perms: ActionsPermissions = serde_json::from_str(json).unwrap();
        assert!(perms.enabled);
        assert_eq!(perms.allowed_actions, Some("all".to_string()));
        assert_eq!(
            perms.default_workflow_permissions,
            Some("write".to_string())
        );
        assert_eq!(perms.can_approve_pull_request_reviews, Some(true));
    }

    #[test]
    fn test_actions_permissions_local_only() {
        let json = r#"{
            "enabled": true,
            "allowed_actions": "local_only"
        }"#;

        let perms: ActionsPermissions = serde_json::from_str(json).unwrap();
        assert!(perms.enabled);
        assert_eq!(perms.allowed_actions, Some("local_only".to_string()));
    }
}
