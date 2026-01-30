//! Configuration loader

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::error::{ConfigError, RepoLensError};

use super::presets::Preset;
use super::{
    ActionsConfig, CustomRulesConfig, RuleConfig, SecretsConfig, TemplatesConfig, UrlConfig,
};

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

    /// Custom rules configuration
    #[serde(default)]
    #[serde(rename = "rules.custom")]
    pub custom_rules: CustomRulesConfig,
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
            custom_rules: CustomRulesConfig::default(),
        }
    }
}

impl Config {
    /// Load configuration from file or return default
    pub fn load_or_default() -> Result<Self, RepoLensError> {
        let config_path = Path::new(CONFIG_FILENAME);

        if config_path.exists() {
            Self::load_from_file(config_path)
        } else {
            Ok(Self::default())
        }
    }

    /// Load configuration from a specific file
    pub fn load_from_file(path: &Path) -> Result<Self, RepoLensError> {
        let content = fs::read_to_string(path).map_err(|e| {
            RepoLensError::Config(ConfigError::FileRead {
                path: path.display().to_string(),
                source: e,
            })
        })?;

        toml::from_str(&content).map_err(Into::into)
    }

    /// Create a new configuration from a preset
    pub fn from_preset(preset: Preset) -> Self {
        let mut config = Self {
            preset: preset.name().to_string(),
            ..Default::default()
        };

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
    pub fn to_toml(&self) -> Result<String, RepoLensError> {
        toml::to_string_pretty(self).map_err(Into::into)
    }

    /// Check if a rule is enabled
    pub fn is_rule_enabled(&self, rule_id: &str) -> bool {
        self.rules.get(rule_id).map(|r| r.enabled).unwrap_or(true)
    }

    /// Get severity override for a rule
    #[allow(dead_code)]
    pub fn get_rule_severity(&self, rule_id: &str) -> Option<&str> {
        self.rules.get(rule_id).and_then(|r| r.severity.as_deref())
    }

    /// Check if a file should be ignored for secrets scanning
    pub fn should_ignore_file(&self, file_path: &str) -> bool {
        self.secrets
            .ignore_files
            .iter()
            .any(|pattern| glob_match(pattern, file_path))
    }

    /// Check if a pattern should be ignored for secrets scanning
    pub fn should_ignore_pattern(&self, path: &str) -> bool {
        self.secrets
            .ignore_patterns
            .iter()
            .any(|pattern| glob_match(pattern, path))
    }

    /// Check if a URL is allowed (for enterprise mode)
    #[allow(dead_code)]
    pub fn is_url_allowed(&self, url: &str) -> bool {
        if self.urls.allowed_internal.is_empty() {
            return false;
        }

        self.urls
            .allowed_internal
            .iter()
            .any(|pattern| glob_match(pattern, url))
    }
}

fn glob_match(pattern: &str, text: &str) -> bool {
    if pattern.contains("**") {
        return glob_match_double_star(pattern, text);
    }

    if pattern.contains('*') {
        return glob_match_single_star(pattern, text);
    }

    text == pattern
}

fn glob_match_double_star(pattern: &str, text: &str) -> bool {
    let parts: Vec<&str> = pattern.split("**").collect();

    if parts.len() == 3 && parts[0].is_empty() && parts[2].is_empty() {
        let middle = parts[1].trim_matches('/');
        return text.contains(&format!("/{}", middle)) || text.starts_with(middle);
    }

    if parts.len() != 2 {
        return false;
    }

    let prefix = parts[0].trim_end_matches('/');
    let suffix_raw = parts[1];
    let suffix = suffix_raw.trim_start_matches('/');

    if !prefix.is_empty() && !text.starts_with(prefix) {
        return false;
    }

    if suffix.is_empty() {
        return true;
    }

    if suffix.starts_with('*') {
        let suffix_pattern = suffix.trim_start_matches('*');
        return text.ends_with(suffix_pattern);
    }

    if prefix.is_empty() {
        if suffix_raw.starts_with('/') {
            let pattern_to_find = format!("/{}", suffix);
            if text.contains(&pattern_to_find) {
                return true;
            }
            if text.starts_with(suffix) {
                return true;
            }
            return false;
        }
        return text.contains(suffix);
    }

    if let Some(after_prefix) = text.strip_prefix(prefix) {
        return after_prefix.contains(suffix) || after_prefix.ends_with(suffix);
    }

    text.ends_with(suffix) || text.contains(suffix)
}

fn glob_match_single_star(pattern: &str, text: &str) -> bool {
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

    if let Some(last_part) = parts.last() {
        if !last_part.is_empty() {
            return text.ends_with(last_part);
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_match() {
        assert!(glob_match("*.ts", "file.ts"));
        assert!(glob_match("*.ts", "path/to/file.ts"));
        assert!(!glob_match("*.ts", "file.js"));
        assert!(
            glob_match("**/test/**", "src/test/file.ts"),
            "Pattern **/test/** should match src/test/file.ts"
        );
        assert!(glob_match("**/test/**", "test/file.ts"));
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

    #[test]
    fn test_custom_rules_config_parsing() {
        let toml_content = r#"
preset = "opensource"

["rules.custom"."no-todo"]
pattern = "TODO"
severity = "warning"
files = ["**/*.rs"]
message = "TODO comment found"
"#;
        let config: Config = toml::from_str(toml_content).unwrap();
        assert!(config.custom_rules.rules.contains_key("no-todo"));
        let rule = config.custom_rules.rules.get("no-todo").unwrap();
        assert_eq!(rule.pattern, Some("TODO".to_string()));
        assert_eq!(rule.severity, "warning");
    }
}
