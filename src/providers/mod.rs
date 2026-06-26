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
//! use repolens::providers::github::GitHubProvider;
//!
//! // Check if gh CLI is available
//! if GitHubProvider::is_available() {
//!     println!("GitHub CLI is available");
//! } else {
//!     eprintln!("GitHub CLI not available");
//! }
//! ```

pub mod github;

use serde::{Deserialize, Serialize};

use crate::error::RepoLensError;

// Re-export the shared, provider-agnostic data types so callers depend on
// `providers::*` rather than `providers::github::*`. The GitHub wire-format
// structs themselves remain defined in `github.rs`.
pub use github::{
    ActionsPermissions, BranchProtection, GitHubProvider, RepoInfo, SecretScanningSettings,
};

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
    fn owner(&self) -> &str;

    /// Repository name / project (e.g. the `name` in `owner/name`).
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
}

/// Construct the [`RepoProvider`] for the current configuration.
///
/// Returns `None` when no provider is available/authenticated, preserving the
/// previous graceful-skip behaviour (callers emit no findings in that case).
pub fn for_config(config: &crate::config::Config) -> Option<Box<dyn RepoProvider>> {
    match config.provider {
        Provider::GitHub => {
            if !GitHubProvider::is_available() {
                return None;
            }
            GitHubProvider::new()
                .ok()
                .map(|p| Box::new(p) as Box<dyn RepoProvider>)
        }
        Provider::GitLab => {
            tracing::debug!("gitlab provider not yet implemented");
            None
        }
    }
}
