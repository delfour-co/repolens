//! Configuration module

pub mod loader;
pub mod presets;

pub use loader::get_env_verbosity;
pub use loader::Config;
pub use presets::Preset;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-export CacheConfig from cache module for convenience
pub use crate::cache::CacheConfig;

// Re-export HooksConfig from hooks module for convenience
pub use crate::hooks::HooksConfig;

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
    /// Regex pattern to match (required if command is not set)
    #[serde(default)]
    pub pattern: Option<String>,

    /// Shell command to execute (required if pattern is not set)
    /// The rule triggers if the command returns exit code 0 (or non-zero if invert=true)
    #[serde(default)]
    pub command: Option<String>,

    /// Severity level (critical, warning, info)
    #[serde(default = "default_custom_severity")]
    pub severity: String,

    /// File glob patterns to include (only used with pattern, not with command)
    #[serde(default)]
    pub files: Vec<String>,

    /// Custom message for the finding
    pub message: Option<String>,

    /// Detailed description
    pub description: Option<String>,

    /// Suggested remediation
    pub remediation: Option<String>,

    /// If true, fail when pattern is NOT found or command returns non-zero (inverted matching)
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

/// License compliance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseComplianceConfig {
    /// Whether license compliance checking is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// List of allowed SPDX license identifiers
    /// If empty, all known licenses are allowed (unless denied)
    #[serde(default)]
    pub allowed_licenses: Vec<String>,

    /// List of denied SPDX license identifiers
    #[serde(default)]
    pub denied_licenses: Vec<String>,
}

impl Default for LicenseComplianceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            allowed_licenses: Vec::new(),
            denied_licenses: Vec::new(),
        }
    }
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
    fn test_secrets_config_default() {
        let config = SecretsConfig::default();
        assert!(config.ignore_patterns.is_empty());
        assert!(config.ignore_files.is_empty());
        assert!(config.custom_patterns.is_empty());
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
    fn test_templates_config_default() {
        let config = TemplatesConfig::default();
        assert!(config.license_author.is_none());
        assert!(config.license_year.is_none());
        assert!(config.project_name.is_none());
        assert!(config.project_description.is_none());
    }

    #[test]
    fn test_custom_rule_deserialize() {
        let toml_str = r#"
            pattern = "TODO|FIXME"
            severity = "warning"
            files = ["*.rs", "*.py"]
            message = "Found TODO comment"
            description = "TODO comments should be addressed"
            remediation = "Complete the task or remove the comment"
            invert = false
        "#;
        let rule: CustomRule = toml::from_str(toml_str).unwrap();
        assert_eq!(rule.pattern, Some("TODO|FIXME".to_string()));
        assert_eq!(rule.severity, "warning");
        assert_eq!(rule.files.len(), 2);
        assert!(!rule.invert);
    }

    #[test]
    fn test_custom_rule_with_command() {
        let toml_str = r#"
            command = "test -f Makefile"
            severity = "info"
            message = "Makefile not found"
            invert = true
        "#;
        let rule: CustomRule = toml::from_str(toml_str).unwrap();
        assert!(rule.pattern.is_none());
        assert_eq!(rule.command, Some("test -f Makefile".to_string()));
        assert!(rule.invert);
    }

    #[test]
    fn test_custom_rules_config_default() {
        let config = CustomRulesConfig::default();
        assert!(config.rules.is_empty());
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
    fn test_default_custom_severity_function() {
        assert_eq!(default_custom_severity(), "warning");
    }

    #[test]
    fn test_license_compliance_config_default() {
        let config = LicenseComplianceConfig::default();
        assert!(config.enabled);
        assert!(config.allowed_licenses.is_empty());
        assert!(config.denied_licenses.is_empty());
    }

    #[test]
    fn test_license_compliance_config_deserialize() {
        let toml_str = r#"
            enabled = true
            allowed_licenses = ["MIT", "Apache-2.0"]
            denied_licenses = ["GPL-3.0"]
        "#;
        let config: LicenseComplianceConfig = toml::from_str(toml_str).unwrap();
        assert!(config.enabled);
        assert_eq!(config.allowed_licenses.len(), 2);
        assert_eq!(config.denied_licenses.len(), 1);
        assert_eq!(config.allowed_licenses[0], "MIT");
        assert_eq!(config.denied_licenses[0], "GPL-3.0");
    }

    #[test]
    fn test_license_compliance_config_deserialize_defaults() {
        let toml_str = r#""#;
        let config: LicenseComplianceConfig = toml::from_str(toml_str).unwrap();
        assert!(config.enabled);
        assert!(config.allowed_licenses.is_empty());
        assert!(config.denied_licenses.is_empty());
    }
}
