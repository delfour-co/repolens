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
                "files/sensitive",
                "files/large",
                "files/gitignore",
                "security/dependencies",
                "workflows/secrets",
                "workflows/permissions",
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
                "github/branch-protection",
                "github/settings",
                "quality/tests",
                "quality/linting",
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
}
