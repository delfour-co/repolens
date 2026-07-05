//! # Providers Module
//!
//! This module handles integrations with external services, primarily GitHub.
//!
//! ## GitHub Integration
//!
//! The [`github`] module provides functionality for:
//!
//! - Repository information retrieval
//! - Branch protection rule management
//! - Repository settings configuration
//! - Authentication via `gh` CLI
//!
//! ## Prerequisites
//!
//! GitHub operations require:
//!
//! 1. GitHub CLI (`gh`) installed and in PATH
//! 2. Authentication via `gh auth login`
//! 3. Appropriate repository permissions
//!
//! ## Examples
//!
//! ### Checking GitHub Authentication
//!
//! ```rust,no_run
//! use repolens_core::providers::github::GitHubProvider;
//!
//! // Check if gh CLI is available
//! if GitHubProvider::is_available() {
//!     println!("GitHub CLI is available");
//! } else {
//!     eprintln!("GitHub CLI not available");
//! }
//! ```

pub mod github;
pub mod gitlab;

use serde::{Deserialize, Serialize};

use crate::error::RepoLensError;

// Re-export the shared, provider-agnostic data types so callers depend on
// `providers::*` rather than `providers::github::*`. The GitHub wire-format
// structs themselves remain defined in `github.rs`.
pub use github::{
    ActionsPermissions, BranchProtection, GitHubProvider, RepoInfo, SecretScanningSettings,
};
pub use gitlab::GitLabProvider;

/// Repository hosting provider.
///
/// Selects which [`RepoProvider`] implementation is constructed by
/// [`for_config`]. Defaults to [`Provider::GitHub`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    /// GitHub (github.com or GitHub Enterprise).
    #[default]
    GitHub,
    /// GitLab (gitlab.com or self-managed). Not yet implemented (see Plan B2).
    GitLab,
}

/// Repository metadata fetched from the provider (description, topics, homepage).
///
/// This is the provider-agnostic shape consumed by the `metadata` rule
/// category. GitHub populates it via `gh repo view`.
#[derive(Debug, Deserialize)]
pub struct RepoMetadata {
    /// Repository description, if set.
    pub description: Option<String>,
    /// Topics / tags configured on the repository.
    #[serde(default)]
    pub topics: Vec<String>,
    /// Configured website / homepage URL, if any.
    pub homepage: Option<String>,
    /// Whether GitHub Pages is enabled (GitHub-specific; currently unused).
    #[serde(rename = "hasPages")]
    #[allow(dead_code)]
    pub has_pages: Option<bool>,
}

/// Read-side abstraction over a repository hosting provider.
///
/// Each method mirrors the existing read performed by the kept rules and the
/// action planner. Implementations preserve the current graceful-skip contract:
/// a transient API failure surfaces as an `Err`, which callers translate into
/// "skip this check, emit no finding".
pub trait RepoProvider: Send + Sync {
    /// Repository owner / namespace (e.g. the `owner` in `owner/name`).
    #[allow(dead_code)]
    fn owner(&self) -> &str;

    /// Repository name / project (e.g. the `name` in `owner/name`).
    #[allow(dead_code)]
    fn name(&self) -> &str;

    /// Fetch repository metadata (description, topics, homepage).
    fn repo_metadata(&self) -> Result<RepoMetadata, RepoLensError>;

    /// Branch protection settings for `branch`, or `None` if unprotected.
    fn get_branch_protection(
        &self,
        branch: &str,
    ) -> Result<Option<BranchProtection>, RepoLensError>;

    /// Repository settings (issues / discussions / wiki enablement).
    fn get_repo_settings(&self) -> Result<RepoInfo, RepoLensError>;

    /// Whether vulnerability alerts are enabled.
    fn has_vulnerability_alerts(&self) -> Result<bool, RepoLensError>;

    /// Whether automated security fixes are enabled.
    fn has_automated_security_fixes(&self) -> Result<bool, RepoLensError>;

    /// Whether Dependabot security updates are enabled.
    fn has_dependabot_security_updates(&self) -> Result<bool, RepoLensError>;

    /// Secret scanning / push-protection settings.
    fn get_secret_scanning(&self) -> Result<SecretScanningSettings, RepoLensError>;

    /// Actions permissions (allowed actions, etc.).
    fn get_actions_permissions(&self) -> Result<ActionsPermissions, RepoLensError>;

    /// Default workflow permissions for the provider's CI token.
    fn get_actions_workflow_permissions(&self) -> Result<ActionsPermissions, RepoLensError>;

    /// Whether fork pull-request workflows require approval.
    fn get_fork_pr_workflows_policy(&self) -> Result<bool, RepoLensError>;

    // --- Write side (drives `apply`) ---

    /// Apply protected-branch settings to `branch`.
    fn set_protected_branch(
        &self,
        branch: &str,
        settings: &crate::actions::plan::BranchProtectionSettings,
    ) -> Result<(), RepoLensError>;

    /// Apply repository settings (issues / discussions / wiki / security toggles).
    fn set_repo_settings(
        &self,
        settings: &crate::actions::plan::GitHubRepoSettings,
    ) -> Result<(), RepoLensError>;

    /// Apply repository metadata (description, topics, homepage).
    fn set_repo_metadata(
        &self,
        description: Option<&str>,
        topics: &[String],
        homepage: Option<&str>,
    ) -> Result<(), RepoLensError>;

    /// Create an issue in the repository, returning its URL.
    ///
    /// Foundation for review bug #8: `apply --create-pr`'s issue-creation path
    /// used to hardcode `GitHubProvider`, orphaning the request on GitLab.
    /// Exposing this on the trait lets callers go through `&dyn RepoProvider`
    /// instead (the bin-side rewire is a separate task).
    fn create_issue(
        &self,
        title: &str,
        body: &str,
        labels: &[&str],
    ) -> Result<String, RepoLensError>;

    /// Open a change request (a pull request on GitHub, a merge request on
    /// GitLab) from `head` into `base` (or the repository's default branch
    /// when `base` is `None`), returning its URL.
    fn open_change_request(
        &self,
        title: &str,
        body: &str,
        head: &str,
        base: Option<&str>,
    ) -> Result<String, RepoLensError>;
}

/// Construct the [`RepoProvider`] for the current configuration.
///
/// Returns `None` when no provider is available/authenticated, preserving the
/// previous graceful-skip behaviour (callers emit no findings in that case).
pub fn for_config(config: &crate::config::Config) -> Option<Box<dyn RepoProvider>> {
    let provider = resolve_provider(config.provider, detect_provider_from_remote());

    match provider {
        Provider::GitHub => {
            if !GitHubProvider::is_available() {
                return None;
            }
            GitHubProvider::new()
                .ok()
                .map(|p| Box::new(p) as Box<dyn RepoProvider>)
        }
        Provider::GitLab => {
            if !GitLabProvider::is_available() {
                return None;
            }
            GitLabProvider::new()
                .ok()
                .map(|p| Box::new(p) as Box<dyn RepoProvider>)
        }
    }
}

/// Auto-detect the hosting provider from the git `origin` remote host.
///
/// `github.com` → [`Provider::GitHub`]; `gitlab.com` or any host containing
/// `gitlab` (self-managed) → [`Provider::GitLab`]. Returns `None` when there is
/// no remote or the host is unrecognised, so the caller can fall back to the
/// default.
pub fn detect_provider_from_remote() -> Option<Provider> {
    crate::utils::prerequisites::detect_remote_host().and_then(|host| provider_for_host(&host))
}

/// Resolve the effective provider from an explicit configuration choice and an
/// auto-detected fallback.
///
/// An explicit `configured` choice (from `.repolens.toml`'s `provider` key or
/// the `--provider` CLI flag, carried as `Some(_)`) always wins — including an
/// explicit `Provider::GitHub`, which used to be indistinguishable from
/// "unset" (review bug #5). Auto-detection from the `origin` remote host only
/// applies when `configured` is `None`; absent both, [`Provider::GitHub`] is
/// the default.
fn resolve_provider(configured: Option<Provider>, detected: Option<Provider>) -> Provider {
    configured.unwrap_or_else(|| detected.unwrap_or(Provider::GitHub))
}

/// Map a remote host string to a [`Provider`].
fn provider_for_host(host: &str) -> Option<Provider> {
    let host = host.to_ascii_lowercase();
    if host.contains("github.com") {
        Some(Provider::GitHub)
    } else if host.contains("gitlab") {
        Some(Provider::GitLab)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_for_host_github() {
        assert_eq!(provider_for_host("github.com"), Some(Provider::GitHub));
        assert_eq!(provider_for_host("GitHub.com"), Some(Provider::GitHub));
    }

    #[test]
    fn test_provider_for_host_gitlab() {
        assert_eq!(provider_for_host("gitlab.com"), Some(Provider::GitLab));
        assert_eq!(
            provider_for_host("gitlab.internal.corp"),
            Some(Provider::GitLab)
        );
    }

    #[test]
    fn test_provider_for_host_unknown() {
        assert_eq!(provider_for_host("bitbucket.org"), None);
        assert_eq!(provider_for_host("example.com"), None);
    }

    #[test]
    fn test_detect_provider_from_remote_does_not_panic() {
        let _ = detect_provider_from_remote();
    }

    /// Regression test for review bug #5: an explicit `--provider github`
    /// selection (or `provider = "github"` in `.repolens.toml`) must win even
    /// when the git remote auto-detects as GitLab.
    #[test]
    fn test_explicit_provider_github_not_overridden_by_autodetect() {
        assert_eq!(
            resolve_provider(Some(Provider::GitHub), Some(Provider::GitLab)),
            Provider::GitHub
        );
    }

    #[test]
    fn test_explicit_provider_gitlab_not_overridden_by_autodetect() {
        assert_eq!(
            resolve_provider(Some(Provider::GitLab), Some(Provider::GitHub)),
            Provider::GitLab
        );
    }

    #[test]
    fn test_unset_provider_falls_back_to_autodetect_then_github() {
        assert_eq!(
            resolve_provider(None, Some(Provider::GitLab)),
            Provider::GitLab
        );
        assert_eq!(resolve_provider(None, None), Provider::GitHub);
    }
}
