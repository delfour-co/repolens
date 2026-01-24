//! Configuration loader

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::{
    ActionsConfig, RuleConfig, SecretsConfig, TemplatesConfig, UrlConfig,
};
use super::presets::Preset;

const CONFIG_FILENAME: &str = ".repolens.toml";

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Preset name (opensource, enterprise, strict)
    #[serde(default = "default_preset")]
    pub preset: String,

    /// Rule overrides
    #[serde(default)]
    pub rules: HashMap<String, RuleConfig>,

    /// Secrets detection configuration
    #[serde(default)]
    #[serde(rename = "rules.secrets")]
    pub secrets: SecretsConfig,

    /// URL detection configuration
    #[serde(default)]
    #[serde(rename = "rules.urls")]
    pub urls: UrlConfig,

    /// Actions configuration
    #[serde(default)]
    pub actions: ActionsConfig,

    /// Template configuration
    #[serde(default)]
    pub templates: TemplatesConfig,
}

fn default_preset() -> String {
    "opensource".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            preset: "opensource".to_string(),
            rules: HashMap::new(),
            secrets: SecretsConfig::default(),
            urls: UrlConfig::default(),
            actions: ActionsConfig::default(),
            templates: TemplatesConfig::default(),
        }
    }
}

impl Config {
    /// Load configuration from file or return default
    pub fn load_or_default() -> Result<Self> {
        let config_path = Path::new(CONFIG_FILENAME);

        if config_path.exists() {
            Self::load_from_file(config_path)
        } else {
            Ok(Self::default())
        }
    }

    /// Load configuration from a specific file
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .context("Failed to read configuration file")?;

        toml::from_str(&content)
            .context("Failed to parse configuration file")
    }

    /// Create a new configuration from a preset
    pub fn from_preset(preset: Preset) -> Self {
        let mut config = Self::default();
        config.preset = preset.name().to_string();

        // Apply preset-specific defaults
        match preset {
            Preset::OpenSource => {
                config.actions.license.enabled = true;
                config.actions.contributing = true;
                config.actions.code_of_conduct = true;
                config.actions.security_policy = true;
                config.actions.github_settings.discussions = true;
            }
            Preset::Enterprise => {
                config.actions.license.enabled = false;
                config.actions.contributing = false;
                config.actions.code_of_conduct = false;
                config.actions.security_policy = true;
                config.actions.branch_protection.required_approvals = 2;
                config.actions.branch_protection.require_signed_commits = true;
                config.actions.github_settings.discussions = false;
            }
            Preset::Strict => {
                config.actions.license.enabled = true;
                config.actions.contributing = true;
                config.actions.code_of_conduct = true;
                config.actions.security_policy = true;
                config.actions.branch_protection.required_approvals = 2;
                config.actions.branch_protection.require_signed_commits = true;
                config.actions.github_settings.discussions = true;
            }
        }

        config
    }

    /// Serialize configuration to TOML
    pub fn to_toml(&self) -> Result<String> {
        toml::to_string_pretty(self)
            .context("Failed to serialize configuration")
    }

    /// Check if a rule is enabled
    pub fn is_rule_enabled(&self, rule_id: &str) -> bool {
        self.rules
            .get(rule_id)
            .map(|r| r.enabled)
            .unwrap_or(true)
    }

    /// Get severity override for a rule
    pub fn get_rule_severity(&self, rule_id: &str) -> Option<&str> {
        self.rules
            .get(rule_id)
            .and_then(|r| r.severity.as_deref())
    }

    /// Check if a file should be ignored for secrets scanning
    pub fn should_ignore_file(&self, file_path: &str) -> bool {
        self.secrets.ignore_files.iter().any(|pattern| {
            glob_match(pattern, file_path)
        })
    }

    /// Check if a pattern should be ignored for secrets scanning
    pub fn should_ignore_pattern(&self, path: &str) -> bool {
        self.secrets.ignore_patterns.iter().any(|pattern| {
            glob_match(pattern, path)
        })
    }

    /// Check if a URL is allowed (for enterprise mode)
    pub fn is_url_allowed(&self, url: &str) -> bool {
        if self.urls.allowed_internal.is_empty() {
            return false;
        }

        self.urls.allowed_internal.iter().any(|pattern| {
            glob_match(pattern, url)
        })
    }
}

/// Simple glob matching (supports * and **)
fn glob_match(pattern: &str, text: &str) -> bool {
    if pattern.contains("**") {
        // For ** patterns, use simple contains check
        let parts: Vec<&str> = pattern.split("**").collect();
        if parts.len() == 2 {
            let prefix = parts[0].trim_end_matches('/');
            let suffix = parts[1].trim_start_matches('/');

            if !prefix.is_empty() && !text.starts_with(prefix) {
                return false;
            }
            if !suffix.is_empty() {
                // Handle patterns like *.test.ts by checking if text ends with the suffix
                // after handling the leading * in the suffix
                if suffix.starts_with('*') {
                    let suffix_pattern = suffix.trim_start_matches('*');
                    if !text.ends_with(suffix_pattern) {
                        return false;
                    }
                } else if !text.ends_with(suffix) && !text.contains(&format!("/{}", suffix)) {
                    return false;
                }
            }
            return true;
        }
    }

    if pattern.contains('*') {
        // Simple * wildcard matching
        let parts: Vec<&str> = pattern.split('*').collect();
        let mut pos = 0;

        for (i, part) in parts.iter().enumerate() {
            if part.is_empty() {
                continue;
            }

            if let Some(found_pos) = text[pos..].find(part) {
                if i == 0 && found_pos != 0 {
                    return false;
                }
                pos += found_pos + part.len();
            } else {
                return false;
            }
        }

        if !parts.last().unwrap_or(&"").is_empty() {
            return text.ends_with(parts.last().unwrap());
        }

        return true;
    }

    text == pattern
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_match() {
        assert!(glob_match("*.ts", "file.ts"));
        assert!(glob_match("*.ts", "path/to/file.ts"));
        assert!(!glob_match("*.ts", "file.js"));

        assert!(glob_match("**/test/**", "src/test/file.ts"));
        assert!(glob_match("**/*.test.ts", "src/file.test.ts"));
    }

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.preset, "opensource");
        assert!(config.actions.gitignore);
    }

    #[test]
    fn test_from_preset() {
        let config = Config::from_preset(Preset::Enterprise);
        assert_eq!(config.preset, "enterprise");
        assert!(!config.actions.license.enabled);
        assert_eq!(config.actions.branch_protection.required_approvals, 2);
    }
}
