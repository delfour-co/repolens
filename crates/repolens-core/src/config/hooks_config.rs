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
