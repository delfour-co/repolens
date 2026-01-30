//! Preset configurations for different use cases

/// Available presets
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Preset {
    /// Open source project - prepare for public release
    OpenSource,
    /// Enterprise project - internal company standards
    Enterprise,
    /// Strict mode - maximum security and compliance
    Strict,
}

impl Preset {
    /// Get preset from name
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "opensource" | "open-source" | "oss" => Some(Self::OpenSource),
            "enterprise" | "ent" | "internal" => Some(Self::Enterprise),
            "strict" | "secure" | "compliance" => Some(Self::Strict),
            _ => None,
        }
    }

    /// Get the name of the preset
    pub fn name(&self) -> &'static str {
        match self {
            Self::OpenSource => "opensource",
            Self::Enterprise => "enterprise",
            Self::Strict => "strict",
        }
    }

    /// Get a description of the preset
    #[allow(dead_code)]
    pub fn description(&self) -> &'static str {
        match self {
            Self::OpenSource => "Prepare repository for public open source release",
            Self::Enterprise => "Apply internal company standards and policies",
            Self::Strict => "Maximum security and compliance checks",
        }
    }

    /// Get the rules that should be enabled for this preset
    #[allow(dead_code)]
    pub fn enabled_rules(&self) -> Vec<&'static str> {
        match self {
            Self::OpenSource => vec![
                "secrets/hardcoded",
                "secrets/files",
                "secrets/env",
                "docs/readme",
                "docs/license",
                "docs/contributing",
                "docs/code-of-conduct",
                "docs/security",
                "docs/changelog",
                "files/sensitive",
                "files/large",
                "files/gitignore",
                "security/dependencies",
                "workflows/secrets",
                "workflows/permissions",
                "workflows/linters-in-ci",
                "docker/dockerfile-presence",
                "docker/dockerignore",
                "docker/from-pinning",
                "docker/user",
                "github/branch-protection",
                "github/settings",
            ],
            Self::Enterprise => vec![
                "secrets/hardcoded",
                "secrets/files",
                "secrets/env",
                "docs/readme",
                "docs/security",
                "files/sensitive",
                "files/large",
                "files/gitignore",
                "security/dependencies",
                "security/codeowners",
                "security/signed-commits",
                "workflows/secrets",
                "workflows/permissions",
                "workflows/timeout",
                "workflows/pull-request-target",
                "docker/dockerfile-presence",
                "docker/dockerignore",
                "docker/from-pinning",
                "docker/user",
                "docker/secrets-in-env",
                "docker/healthcheck",
                "quality/coverage",
                "github/branch-protection",
                "github/settings",
            ],
            Self::Strict => vec![
                "secrets/hardcoded",
                "secrets/files",
                "secrets/env",
                "secrets/history",
                "docs/readme",
                "docs/license",
                "docs/contributing",
                "docs/code-of-conduct",
                "docs/security",
                "docs/changelog",
                "docs/changelog-format",
                "docs/changelog-unreleased",
                "files/sensitive",
                "files/large",
                "files/gitignore",
                "files/editorconfig",
                "security/dependencies",
                "security/codeowners",
                "security/signed-commits",
                "workflows/secrets",
                "workflows/permissions",
                "workflows/pinned-actions",
                "workflows/timeout",
                "workflows/concurrency",
                "workflows/reusable-workflows",
                "workflows/artifacts-retention",
                "workflows/pull-request-target",
                "workflows/linters-in-ci",
                "docker/dockerfile-presence",
                "docker/dockerignore",
                "docker/from-pinning",
                "docker/user",
                "docker/healthcheck",
                "docker/multistage",
                "docker/secrets-in-env",
                "docker/copy-all",
                "quality/tests",
                "quality/linting",
                "quality/coverage",
                "quality/api-docs",
                "quality/complexity",
                "quality/dead-code",
                "quality/naming-conventions",
                "github/branch-protection",
                "github/settings",
            ],
        }
    }

    /// Get rules with critical severity for this preset
    #[allow(dead_code)]
    pub fn critical_rules(&self) -> Vec<&'static str> {
        match self {
            Self::OpenSource => vec!["secrets/hardcoded", "secrets/files", "docs/license"],
            Self::Enterprise => vec!["secrets/hardcoded", "secrets/files", "security/codeowners"],
            Self::Strict => vec![
                "secrets/hardcoded",
                "secrets/files",
                "secrets/history",
                "docs/license",
                "security/codeowners",
                "security/signed-commits",
                "docker/from-pinning",
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preset_from_name() {
        assert_eq!(Preset::from_name("opensource").unwrap(), Preset::OpenSource);
        assert_eq!(Preset::from_name("oss").unwrap(), Preset::OpenSource);
        assert_eq!(Preset::from_name("enterprise").unwrap(), Preset::Enterprise);
        assert_eq!(Preset::from_name("strict").unwrap(), Preset::Strict);
        assert!(Preset::from_name("invalid").is_none());
    }

    #[test]
    fn test_preset_name() {
        assert_eq!(Preset::OpenSource.name(), "opensource");
        assert_eq!(Preset::Enterprise.name(), "enterprise");
        assert_eq!(Preset::Strict.name(), "strict");
    }

    #[test]
    fn test_preset_from_name_aliases() {
        // OpenSource aliases
        assert_eq!(
            Preset::from_name("open-source").unwrap(),
            Preset::OpenSource
        );
        assert_eq!(Preset::from_name("OPENSOURCE").unwrap(), Preset::OpenSource);

        // Enterprise aliases
        assert_eq!(Preset::from_name("ent").unwrap(), Preset::Enterprise);
        assert_eq!(Preset::from_name("internal").unwrap(), Preset::Enterprise);
        assert_eq!(Preset::from_name("ENTERPRISE").unwrap(), Preset::Enterprise);

        // Strict aliases
        assert_eq!(Preset::from_name("secure").unwrap(), Preset::Strict);
        assert_eq!(Preset::from_name("compliance").unwrap(), Preset::Strict);
        assert_eq!(Preset::from_name("STRICT").unwrap(), Preset::Strict);
    }

    #[test]
    fn test_preset_description() {
        assert!(Preset::OpenSource.description().contains("open source"));
        assert!(Preset::Enterprise.description().contains("internal"));
        assert!(Preset::Strict.description().contains("security"));
    }

    #[test]
    fn test_preset_enabled_rules_opensource() {
        let rules = Preset::OpenSource.enabled_rules();
        assert!(rules.contains(&"secrets/hardcoded"));
        assert!(rules.contains(&"docs/readme"));
        assert!(rules.contains(&"docs/license"));
        assert!(rules.contains(&"docs/contributing"));
        assert!(rules.contains(&"docs/code-of-conduct"));
    }

    #[test]
    fn test_preset_enabled_rules_enterprise() {
        let rules = Preset::Enterprise.enabled_rules();
        assert!(rules.contains(&"secrets/hardcoded"));
        assert!(rules.contains(&"security/codeowners"));
        assert!(rules.contains(&"security/signed-commits"));
        // Enterprise doesn't require license
        assert!(!rules.contains(&"docs/license"));
    }

    #[test]
    fn test_preset_enabled_rules_strict() {
        let rules = Preset::Strict.enabled_rules();
        assert!(rules.contains(&"secrets/hardcoded"));
        assert!(rules.contains(&"secrets/history"));
        assert!(rules.contains(&"quality/tests"));
        assert!(rules.contains(&"quality/linting"));
        assert!(rules.contains(&"workflows/pinned-actions"));
        // New rules
        assert!(rules.contains(&"docker/from-pinning"));
        assert!(rules.contains(&"workflows/timeout"));
        assert!(rules.contains(&"quality/coverage"));
        assert!(rules.contains(&"docs/changelog-format"));
    }

    #[test]
    fn test_preset_enabled_rules_opensource_new() {
        let rules = Preset::OpenSource.enabled_rules();
        assert!(rules.contains(&"docs/changelog"));
        assert!(rules.contains(&"workflows/linters-in-ci"));
        assert!(rules.contains(&"docker/dockerfile-presence"));
    }

    #[test]
    fn test_preset_enabled_rules_enterprise_new() {
        let rules = Preset::Enterprise.enabled_rules();
        assert!(rules.contains(&"workflows/timeout"));
        assert!(rules.contains(&"workflows/pull-request-target"));
        assert!(rules.contains(&"docker/from-pinning"));
        assert!(rules.contains(&"quality/coverage"));
    }

    #[test]
    fn test_preset_critical_rules_strict_docker() {
        let rules = Preset::Strict.critical_rules();
        assert!(rules.contains(&"docker/from-pinning"));
    }

    #[test]
    fn test_preset_critical_rules_opensource() {
        let rules = Preset::OpenSource.critical_rules();
        assert!(rules.contains(&"secrets/hardcoded"));
        assert!(rules.contains(&"secrets/files"));
        assert!(rules.contains(&"docs/license"));
    }

    #[test]
    fn test_preset_critical_rules_enterprise() {
        let rules = Preset::Enterprise.critical_rules();
        assert!(rules.contains(&"secrets/hardcoded"));
        assert!(rules.contains(&"security/codeowners"));
    }

    #[test]
    fn test_preset_critical_rules_strict() {
        let rules = Preset::Strict.critical_rules();
        assert!(rules.contains(&"secrets/hardcoded"));
        assert!(rules.contains(&"secrets/history"));
        assert!(rules.contains(&"security/signed-commits"));
    }

    #[test]
    fn test_preset_equality() {
        assert_eq!(Preset::OpenSource, Preset::OpenSource);
        assert_ne!(Preset::OpenSource, Preset::Enterprise);
        assert_ne!(Preset::Enterprise, Preset::Strict);
    }

    #[test]
    fn test_preset_copy() {
        let preset = Preset::OpenSource;
        let copied = preset;
        assert_eq!(preset, copied);
    }
}
