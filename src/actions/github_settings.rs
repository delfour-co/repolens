//! Repository settings management via the provider abstraction.

use crate::config::Config;
use crate::error::{ActionError, ProviderError, RepoLensError};

use super::plan::GitHubRepoSettings;

/// Update repository settings.
///
/// Routes through the configured [`crate::providers::RepoProvider`] rather than
/// shelling out to `gh` directly. When no provider is available/authenticated,
/// the action fails with a clear error.
pub async fn update(config: &Config, settings: &GitHubRepoSettings) -> Result<(), RepoLensError> {
    let provider = crate::providers::for_config(config)
        .ok_or(RepoLensError::Provider(ProviderError::GitHubCliNotAvailable))?;

    provider.set_repo_settings(settings).map_err(|e| match e {
        RepoLensError::Provider(_) | RepoLensError::Action(_) => e,
        other => RepoLensError::Action(ActionError::ExecutionFailed {
            message: format!("Failed to update repository settings: {other}"),
        }),
    })
}
