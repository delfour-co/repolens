//! Configuration module

pub mod loader;
pub mod presets;

pub use loader::Config;
pub use presets::Preset;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Rule configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuleConfig {
    /// Whether the rule is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Severity override (critical, warning, info)
    pub severity: Option<String>,
}

fn default_true() -> bool {
    true
}

/// Secrets configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SecretsConfig {
    /// Patterns to ignore when scanning for secrets
    #[serde(default)]
    pub ignore_patterns: Vec<String>,

    /// Files to ignore when scanning for secrets
    #[serde(default)]
    pub ignore_files: Vec<String>,

    /// Custom secret patterns to detect
    #[serde(default)]
    pub custom_patterns: Vec<String>,
}

/// URL configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UrlConfig {
    /// Allowed internal URLs (for enterprise mode)
    #[serde(default)]
    pub allowed_internal: Vec<String>,
}

/// Actions configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionsConfig {
    /// Whether to update .gitignore
    #[serde(default = "default_true")]
    pub gitignore: bool,

    /// License configuration
    #[serde(default)]
    pub license: LicenseConfig,

    /// Whether to create CONTRIBUTING.md
    #[serde(default = "default_true")]
    pub contributing: bool,

    /// Whether to create CODE_OF_CONDUCT.md
    #[serde(default = "default_true")]
    pub code_of_conduct: bool,

    /// Whether to create SECURITY.md
    #[serde(default = "default_true")]
    pub security_policy: bool,

    /// Branch protection configuration
    #[serde(default)]
    pub branch_protection: BranchProtectionConfig,

    /// GitHub settings configuration
    #[serde(default)]
    pub github_settings: GitHubSettingsConfig,
}

impl Default for ActionsConfig {
    fn default() -> Self {
        Self {
            gitignore: true,
            license: LicenseConfig::default(),
            contributing: true,
            code_of_conduct: true,
            security_policy: true,
            branch_protection: BranchProtectionConfig::default(),
            github_settings: GitHubSettingsConfig::default(),
        }
    }
}

/// License configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseConfig {
    /// Whether to create LICENSE file
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// License type (MIT, Apache-2.0, GPL-3.0, etc.)
    #[serde(default = "default_license_type")]
    pub license_type: String,

    /// Author name for license
    #[serde(default)]
    pub author: Option<String>,

    /// Year for license (defaults to current year)
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

/// Branch protection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchProtectionConfig {
    /// Whether to enable branch protection
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Branch to protect (defaults to main)
    #[serde(default = "default_branch")]
    pub branch: String,

    /// Number of required approvals
    #[serde(default = "default_approvals")]
    pub required_approvals: u32,

    /// Whether to require status checks
    #[serde(default = "default_true")]
    pub require_status_checks: bool,

    /// Whether to block force pushes
    #[serde(default = "default_true")]
    pub block_force_push: bool,

    /// Whether to require signed commits
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

/// GitHub settings configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubSettingsConfig {
    /// Whether to enable GitHub Discussions
    #[serde(default = "default_true")]
    pub discussions: bool,

    /// Whether to enable GitHub Issues
    #[serde(default = "default_true")]
    pub issues: bool,

    /// Whether to enable GitHub Wiki
    #[serde(default)]
    pub wiki: bool,

    /// Whether to enable vulnerability alerts
    #[serde(default = "default_true")]
    pub vulnerability_alerts: bool,

    /// Whether to enable automatic security fixes
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

/// Template configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemplatesConfig {
    /// Author name for templates
    pub license_author: Option<String>,

    /// Year for templates
    pub license_year: Option<String>,

    /// Project name override
    pub project_name: Option<String>,

    /// Project description
    pub project_description: Option<String>,
}

/// Custom rule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomRule {
    /// Regex pattern to match
    pub pattern: String,

    /// Severity level (critical, warning, info)
    #[serde(default = "default_custom_severity")]
    pub severity: String,

    /// File glob patterns to include
    #[serde(default)]
    pub files: Vec<String>,

    /// Custom message for the finding
    pub message: Option<String>,

    /// Detailed description
    pub description: Option<String>,

    /// Suggested remediation
    pub remediation: Option<String>,

    /// If true, fail when pattern is NOT found (inverted matching)
    #[serde(default)]
    pub invert: bool,
}

fn default_custom_severity() -> String {
    "warning".to_string()
}

/// Custom rules configuration container
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CustomRulesConfig {
    /// Map of rule ID to rule configuration
    #[serde(flatten)]
    pub rules: HashMap<String, CustomRule>,
}
