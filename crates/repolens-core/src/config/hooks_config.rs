use serde::{Deserialize, Serialize};

/// Configuration for RepoLens-managed Git hooks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HooksConfig {
    /// Whether to install the pre-commit hook
    #[serde(default = "default_true")]
    pub pre_commit: bool,
    /// Whether to install the pre-push hook
    #[serde(default = "default_true")]
    pub pre_push: bool,
    /// Whether warnings should cause hook failure
    #[serde(default)]
    pub fail_on_warnings: bool,
}

fn default_true() -> bool {
    true
}

impl Default for HooksConfig {
    fn default() -> Self {
        Self {
            pre_commit: true,
            pre_push: true,
            fail_on_warnings: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hooks_config_serialize_deserialize() {
        let config = HooksConfig {
            pre_commit: true,
            pre_push: false,
            fail_on_warnings: true,
        };
        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: HooksConfig = toml::from_str(&toml_str).unwrap();
        assert!(deserialized.pre_commit);
        assert!(!deserialized.pre_push);
        assert!(deserialized.fail_on_warnings);
    }

    #[test]
    fn test_hooks_config_deserialize_from_toml() {
        let toml_str = r#"
            pre_commit = false
            pre_push = true
            fail_on_warnings = true
        "#;
        let config: HooksConfig = toml::from_str(toml_str).unwrap();
        assert!(!config.pre_commit);
        assert!(config.pre_push);
        assert!(config.fail_on_warnings);
    }

    #[test]
    fn test_hooks_config_default_serde() {
        // Verify that default values work when fields are missing from TOML
        let toml_str = "";
        let config: HooksConfig = toml::from_str(toml_str).unwrap();
        assert!(config.pre_commit);
        assert!(config.pre_push);
        assert!(!config.fail_on_warnings);
    }
}
