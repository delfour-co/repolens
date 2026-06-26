//! Branch protection configuration via the provider abstraction.

use crate::config::Config;
use crate::error::{ActionError, ProviderError, RepoLensError};

use super::plan::BranchProtectionSettings;

/// Configure protected-branch settings for a branch.
///
/// Routes through the configured [`crate::providers::RepoProvider`] rather than
/// shelling out to `gh` directly. When no provider is available/authenticated,
/// the action fails with a clear error (as before, when `gh` was unavailable).
pub async fn configure(
    config: &Config,
    branch: &str,
    settings: &BranchProtectionSettings,
) -> Result<(), RepoLensError> {
    let provider = crate::providers::for_config(config)
        .ok_or(RepoLensError::Provider(ProviderError::GitHubCliNotAvailable))?;

    provider.set_protected_branch(branch, settings).map_err(|e| {
        // Preserve a clear action-level failure for non-provider errors.
        match e {
            RepoLensError::Provider(_) | RepoLensError::Action(_) => e,
            other => RepoLensError::Action(ActionError::ExecutionFailed {
                message: format!("Failed to configure branch protection: {other}"),
            }),
        }
    })
}
