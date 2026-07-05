//! # Configuration Module
//!
//! This module handles all configuration-related functionality for RepoLens,
//! including loading configuration files, managing presets, and providing
//! rule-specific settings.
//!
//! ## Configuration Priority
//!
//! Configuration is loaded with the following priority (highest to lowest):
//!
//! 1. CLI flags (handled by clap)
//! 2. Environment variables (`REPOLENS_*`)
//! 3. Configuration file (`.repolens.toml`)
//! 4. Default values
//!
//! ## Configuration File
//!
//! The default configuration file is `.repolens.toml` in the project root.
//!
//! ```toml
//! preset = "opensource"
//!
//! [actions]
//! gitignore = true
//! contributing = true
//!
//! [actions.license]
//! enabled = true
//! license_type = "MIT"
//! author = "Your Name"
//!
//! [actions.branch_protection]
//! enabled = true
//! required_approvals = 1
//! ```
//!
//! ## Environment Variables
//!
//! | Variable | Description |
//! |----------|-------------|
//! | `REPOLENS_PRESET` | Override preset (opensource, enterprise, strict) |
//! | `REPOLENS_CONFIG` | Path to configuration file |
//! | `REPOLENS_VERBOSE` | Verbosity level (0-3) |
//! | `REPOLENS_NO_CACHE` | Disable caching (true/false) |
//! | `REPOLENS_GITHUB_TOKEN` | GitHub API token |
//!
//! ## Examples
//!
//! ### Loading Configuration
//!
//! ```rust,no_run
//! use repolens_core::config::Config;
//!
//! // Load from default location or environment
//! let config = Config::load_or_default().expect("Failed to load config");
//!
//! // Check preset
//! println!("Using preset: {}", config.preset);
//! ```
//!
//! ### Creating from Preset
//!
//! ```rust
//! use repolens_core::config::{Config, Preset};
//!
//! let config = Config::from_preset(Preset::Enterprise);
//! assert_eq!(config.preset, "enterprise");
//! ```
//!
//! ### Checking Rule Configuration
//!
//! ```rust
//! use repolens_core::config::Config;
//!
//! let config = Config::default();
//!
//! // Check if a rule is enabled
//! if config.is_rule_enabled("SEC001") {
//!     println!("Secret detection is enabled");
//! }
//! ```

mod hooks_config;
pub mod loader;
pub mod presets;

pub use loader::Config;
pub use loader::get_env_verbosity;
pub use presets::Preset;

use serde::{Deserialize, Serialize};

// Re-export CacheConfig from cache module for convenience
pub use crate::cache::CacheConfig;

// Re-export HooksConfig for convenience
pub use hooks_config::HooksConfig;

/// Configuration for individual audit rules.
///
/// Allows enabling/disabling rules and overriding their default severity.
///
/// # Examples
///
/// ```toml
/// [rules.SEC001]
/// enabled = false
/// severity = "warning"
/// ```
///
/// ```rust
/// use repolens_core::config::RuleConfig;
///
/// let rule = RuleConfig {
///     enabled: true,
///     severity: Some("critical".to_string()),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuleConfig {
    /// Whether the rule is enabled. Defaults to `true`.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Severity override (critical, warning, info).
    /// If `None`, the rule's default severity is used.
    pub severity: Option<String>,
}

fn default_true() -> bool {
    true
}

/// Configuration for URL validation.
///
/// Used primarily in enterprise mode to allow internal URLs
/// that would otherwise be flagged as potential issues.
///
/// # Examples
///
/// ```toml
/// ["rules.urls"]
/// allowed_internal = ["https://internal.company.com/*", "http://localhost:*"]
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UrlConfig {
    /// Allowed internal URLs (for enterprise mode).
    /// Supports glob patterns for URL matching.
    #[serde(default)]
    pub allowed_internal: Vec<String>,
}

/// Configuration for remediation actions.
///
/// Controls which automated fixes and file generations are enabled
/// when running `repolens apply`.
///
/// # Examples
///
/// ```toml
/// [actions]
/// gitignore = true
/// contributing = true
/// code_of_conduct = true
/// security_policy = true
///
/// [actions.license]
/// enabled = true
/// license_type = "MIT"
///
/// [actions.branch_protection]
/// enabled = true
/// required_approvals = 2
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionsConfig {
    /// Whether to update `.gitignore` with recommended entries.
    #[serde(default = "default_true")]
    pub gitignore: bool,

    /// License file generation configuration.
    #[serde(default)]
    pub license: LicenseConfig,

    /// Whether to create `CONTRIBUTING.md` if missing.
    #[serde(default = "default_true")]
    pub contributing: bool,

    /// Whether to create `CODE_OF_CONDUCT.md` if missing.
    #[serde(default = "default_true")]
    pub code_of_conduct: bool,

    /// Whether to create `SECURITY.md` if missing.
    #[serde(default = "default_true")]
    pub security_policy: bool,

    /// Whether to create `README.md` if missing.
    #[serde(default = "default_true")]
    pub readme: bool,

    /// Whether to create `CHANGELOG.md` if missing.
    #[serde(default = "default_true")]
    pub changelog: bool,

    /// Whether to create `.gitattributes` if missing.
    #[serde(default = "default_true")]
    pub gitattributes: bool,

    /// Whether to create `CODEOWNERS` if missing.
    #[serde(default = "default_true")]
    pub codeowners: bool,

    /// Whether to create `.github/settings.yml` if missing.
    #[serde(default = "default_true")]
    pub settings_file: bool,

    /// GitHub branch protection rule configuration.
    #[serde(default)]
    pub branch_protection: BranchProtectionConfig,

    /// GitHub repository settings configuration.
    #[serde(default)]
    pub github_settings: GitHubSettingsConfig,

    /// Repository metadata configuration (description, topics, homepage).
    #[serde(default)]
    pub metadata: MetadataConfig,

    /// GitHub Actions & repository security settings configuration (secret
    /// scanning, push protection, Actions permissions, workflow
    /// permissions, fork pull-request-workflow approval) -- review bug #11
    /// (SEC013-017).
    #[serde(default)]
    pub actions_security: ActionsSecurityConfig,
}

impl Default for ActionsConfig {
    fn default() -> Self {
        Self {
            gitignore: true,
            license: LicenseConfig::default(),
            contributing: true,
            code_of_conduct: true,
            security_policy: true,
            readme: true,
            changelog: true,
            gitattributes: true,
            codeowners: true,
            settings_file: true,
            branch_protection: BranchProtectionConfig::default(),
            github_settings: GitHubSettingsConfig::default(),
            metadata: MetadataConfig::default(),
            actions_security: ActionsSecurityConfig::default(),
        }
    }
}

/// Configuration for LICENSE file generation.
///
/// Controls the license type and metadata used when generating
/// a LICENSE file.
///
/// # Supported License Types
///
/// - `MIT` - MIT License (default)
/// - `Apache-2.0` - Apache License 2.0
/// - `GPL-3.0` - GNU General Public License v3.0
/// - `BSD-3-Clause` - BSD 3-Clause License
/// - `UNLICENSED` - Proprietary/No License
///
/// # Examples
///
/// ```toml
/// [actions.license]
/// enabled = true
/// license_type = "Apache-2.0"
/// author = "Your Name"
/// year = "2024"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseConfig {
    /// Whether to create LICENSE file if missing.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// License type (MIT, Apache-2.0, GPL-3.0, etc.).
    /// Defaults to "MIT".
    #[serde(default = "default_license_type")]
    pub license_type: String,

    /// Author name for license. If not set, attempts to
    /// detect from git configuration.
    #[serde(default)]
    pub author: Option<String>,

    /// Year for license. Defaults to current year if not specified.
    #[serde(default)]
    pub year: Option<String>,
}

impl Default for LicenseConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            license_type: "MIT".to_string(),
            author: None,
            year: None,
        }
    }
}

fn default_license_type() -> String {
    "MIT".to_string()
}

/// Configuration for GitHub branch protection rules.
///
/// These settings are applied via the GitHub API when running
/// `repolens apply` with appropriate permissions.
///
/// # Examples
///
/// ```toml
/// [actions.branch_protection]
/// enabled = true
/// branch = "main"
/// required_approvals = 2
/// require_status_checks = true
/// block_force_push = true
/// require_signed_commits = true
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchProtectionConfig {
    /// Whether to enable branch protection rules.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Branch to protect. Defaults to "main".
    #[serde(default = "default_branch")]
    pub branch: String,

    /// Number of required pull request approvals.
    /// Defaults to 1, enterprise preset uses 2.
    #[serde(default = "default_approvals")]
    pub required_approvals: u32,

    /// Whether to require status checks to pass before merging.
    #[serde(default = "default_true")]
    pub require_status_checks: bool,

    /// Whether to block force pushes to the protected branch.
    #[serde(default = "default_true")]
    pub block_force_push: bool,

    /// Whether to require signed commits.
    /// Defaults to `false`, enterprise/strict presets enable this.
    #[serde(default)]
    pub require_signed_commits: bool,
}

impl Default for BranchProtectionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            branch: "main".to_string(),
            required_approvals: 1,
            require_status_checks: true,
            block_force_push: true,
            require_signed_commits: false,
        }
    }
}

fn default_branch() -> String {
    "main".to_string()
}

fn default_approvals() -> u32 {
    1
}

/// Configuration for GitHub repository settings.
///
/// These settings are applied via the GitHub API when running
/// `repolens apply` with appropriate permissions.
///
/// # Examples
///
/// ```toml
/// [actions.github_settings]
/// discussions = true
/// issues = true
/// wiki = false
/// vulnerability_alerts = true
/// automated_security_fixes = true
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubSettingsConfig {
    /// Whether to enable GitHub Discussions for the repository.
    #[serde(default = "default_true")]
    pub discussions: bool,

    /// Whether to enable GitHub Issues for the repository.
    #[serde(default = "default_true")]
    pub issues: bool,

    /// Whether to enable GitHub Wiki for the repository.
    /// Defaults to `false` as wikis are often unused.
    #[serde(default)]
    pub wiki: bool,

    /// Whether to enable Dependabot vulnerability alerts.
    #[serde(default = "default_true")]
    pub vulnerability_alerts: bool,

    /// Whether to enable Dependabot automatic security fixes.
    #[serde(default = "default_true")]
    pub automated_security_fixes: bool,
}

impl Default for GitHubSettingsConfig {
    fn default() -> Self {
        Self {
            discussions: true,
            issues: true,
            wiki: false,
            vulnerability_alerts: true,
            automated_security_fixes: true,
        }
    }
}

/// Configuration for repository metadata (description, topics, homepage).
///
/// These values are applied via the provider when running `repolens apply`,
/// but only when the audit reports a missing-metadata finding
/// (META001/002/003) and at least one value is configured here. Metadata
/// cannot be auto-generated from nothing, so — like branch protection and
/// repository settings — this action only applies values the user configured.
///
/// # Examples
///
/// ```toml
/// [actions.metadata]
/// enabled = true
/// description = "A short repository description"
/// topics = ["rust", "cli"]
/// homepage = "https://example.com"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataConfig {
    /// Whether to apply repository metadata when missing.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Repository description to apply, if set.
    #[serde(default)]
    pub description: Option<String>,

    /// Topics / tags to apply, if any.
    #[serde(default)]
    pub topics: Vec<String>,

    /// Homepage / website URL to apply, if set.
    #[serde(default)]
    pub homepage: Option<String>,
}

impl Default for MetadataConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            description: None,
            topics: Vec::new(),
            homepage: None,
        }
    }
}

/// Configuration for GitHub Actions & repository security settings.
///
/// Covers the five GitHub-only toggles behind SEC013-017 (review bug #11):
/// secret scanning, push protection, Actions permissions (which actions are
/// allowed to run), default workflow (`GITHUB_TOKEN`) permissions, and
/// whether fork pull-request workflows require approval before running.
/// These settings are applied via the GitHub API when running `repolens
/// apply` with appropriate permissions. GitHub-only: on GitLab, `apply`
/// skips this action gracefully (no GitLab equivalent exists for any of
/// these five toggles -- `GitLabProvider`'s write methods all return `Err`,
/// and the planner never plans this action for a non-GitHub provider).
///
/// # Examples
///
/// ```toml
/// [actions.actions_security]
/// enabled = true
/// secret_scanning = true
/// push_protection = true
/// allowed_actions = "selected"
/// default_workflow_permissions = "read"
/// require_fork_pr_approval = true
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionsSecurityConfig {
    /// Whether to apply these settings at all.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Whether secret scanning should be enabled (SEC013).
    #[serde(default = "default_true")]
    pub secret_scanning: bool,

    /// Whether push protection should be enabled (SEC014). Only meaningful
    /// once secret scanning itself is enabled.
    #[serde(default = "default_true")]
    pub push_protection: bool,

    /// Desired Actions permissions: `"all"`, `"local_only"`, or `"selected"`
    /// (SEC015). Defaults to `"selected"` (restrict to GitHub-owned and
    /// verified-creator actions).
    #[serde(default = "default_allowed_actions")]
    pub allowed_actions: String,

    /// Desired default `GITHUB_TOKEN` workflow permissions: `"read"` or
    /// `"write"` (SEC016). Defaults to `"read"` (least privilege).
    #[serde(default = "default_workflow_permissions")]
    pub default_workflow_permissions: String,

    /// Whether fork pull request workflows should require approval before
    /// running (SEC017).
    #[serde(default = "default_true")]
    pub require_fork_pr_approval: bool,
}

impl Default for ActionsSecurityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            secret_scanning: true,
            push_protection: true,
            allowed_actions: default_allowed_actions(),
            default_workflow_permissions: default_workflow_permissions(),
            require_fork_pr_approval: true,
        }
    }
}

fn default_allowed_actions() -> String {
    "selected".to_string()
}

fn default_workflow_permissions() -> String {
    "read".to_string()
}

/// Configuration for file template generation.
///
/// These values are used when generating files like LICENSE,
/// CONTRIBUTING.md, and other template-based files.
///
/// # Examples
///
/// ```toml
/// [templates]
/// license_author = "Your Company"
/// license_year = "2024"
/// project_name = "My Project"
/// project_description = "A description of the project"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemplatesConfig {
    /// Author name for license and other templates.
    /// Overrides auto-detected git user name.
    pub license_author: Option<String>,

    /// Year for license templates.
    /// Defaults to current year if not specified.
    pub license_year: Option<String>,

    /// Project name override.
    /// Defaults to repository/directory name if not specified.
    pub project_name: Option<String>,

    /// Project description for generated files.
    pub project_description: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_config_default() {
        let config = RuleConfig::default();
        assert!(!config.enabled); // Default for bool is false
        assert!(config.severity.is_none());
    }

    #[test]
    fn test_rule_config_deserialize() {
        let toml_str = r#"
            enabled = true
            severity = "critical"
        "#;
        let config: RuleConfig = toml::from_str(toml_str).unwrap();
        assert!(config.enabled);
        assert_eq!(config.severity, Some("critical".to_string()));
    }

    #[test]
    fn test_url_config_default() {
        let config = UrlConfig::default();
        assert!(config.allowed_internal.is_empty());
    }

    #[test]
    fn test_actions_config_default() {
        let config = ActionsConfig::default();
        assert!(config.gitignore);
        assert!(config.contributing);
        assert!(config.code_of_conduct);
        assert!(config.security_policy);
    }

    #[test]
    fn test_license_config_default() {
        let config = LicenseConfig::default();
        assert!(config.enabled);
        assert_eq!(config.license_type, "MIT");
        assert!(config.author.is_none());
        assert!(config.year.is_none());
    }

    #[test]
    fn test_branch_protection_config_default() {
        let config = BranchProtectionConfig::default();
        assert!(config.enabled);
        assert_eq!(config.branch, "main");
        assert_eq!(config.required_approvals, 1);
        assert!(config.require_status_checks);
        assert!(config.block_force_push);
        assert!(!config.require_signed_commits);
    }

    #[test]
    fn test_github_settings_config_default() {
        let config = GitHubSettingsConfig::default();
        assert!(config.discussions);
        assert!(config.issues);
        assert!(!config.wiki);
        assert!(config.vulnerability_alerts);
        assert!(config.automated_security_fixes);
    }

    #[test]
    fn test_actions_security_config_default() {
        let config = ActionsSecurityConfig::default();
        assert!(config.enabled);
        assert!(config.secret_scanning);
        assert!(config.push_protection);
        assert_eq!(config.allowed_actions, "selected");
        assert_eq!(config.default_workflow_permissions, "read");
        assert!(config.require_fork_pr_approval);
    }

    #[test]
    fn test_actions_config_default_includes_actions_security() {
        let config = ActionsConfig::default();
        assert!(config.actions_security.enabled);
    }

    #[test]
    fn test_templates_config_default() {
        let config = TemplatesConfig::default();
        assert!(config.license_author.is_none());
        assert!(config.license_year.is_none());
        assert!(config.project_name.is_none());
        assert!(config.project_description.is_none());
    }

    #[test]
    fn test_default_true_function() {
        assert!(default_true());
    }

    #[test]
    fn test_default_license_type_function() {
        assert_eq!(default_license_type(), "MIT");
    }

    #[test]
    fn test_default_branch_function() {
        assert_eq!(default_branch(), "main");
    }

    #[test]
    fn test_default_approvals_function() {
        assert_eq!(default_approvals(), 1);
    }

    #[test]
    fn test_default_allowed_actions_function() {
        assert_eq!(default_allowed_actions(), "selected");
    }

    #[test]
    fn test_default_workflow_permissions_function() {
        assert_eq!(default_workflow_permissions(), "read");
    }
}
