//! Repository metadata management via the provider abstraction.

use crate::config::Config;
use crate::error::{ActionError, ProviderError, RepoLensError};

/// Update repository metadata (description, topics, homepage).
///
/// Routes through the configured [`crate::providers::RepoProvider`]. When no
/// provider is available/authenticated, the action fails with a clear error.
pub async fn update(
    config: &Config,
    description: Option<&str>,
    topics: &[String],
    homepage: Option<&str>,
) -> Result<(), RepoLensError> {
    let provider = crate::providers::for_config(config)
        .ok_or(RepoLensError::Provider(ProviderError::GitHubCliNotAvailable))?;

    provider
        .set_repo_metadata(description, topics, homepage)
        .map_err(|e| match e {
            RepoLensError::Provider(_) | RepoLensError::Action(_) => e,
            other => RepoLensError::Action(ActionError::ExecutionFailed {
                message: format!("Failed to update repository metadata: {other}"),
            }),
        })
}
