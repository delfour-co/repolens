//! GitHub Actions & repository security settings management via the
//! provider abstraction (review bug #11: SEC013-017 genuine remediation).

use crate::config::Config;
use crate::error::{ActionError, ProviderError, RepoLensError};

use super::plan::GitHubActionsSecuritySettings;

/// Apply the requested (`Some`) fields of `settings` via the configured
/// provider.
///
/// Routes through the configured [`crate::providers::RepoProvider`] rather
/// than shelling out to `gh` directly. Each field maps to its own GitHub
/// REST call (see [`crate::providers::RepoProvider::set_secret_scanning`]
/// and its siblings); a `None` field is left untouched. GitHub-only: on
/// GitLab every underlying write returns `Err`, surfaced here as an
/// execution failure -- graceful skip is the *planner*'s job (it never
/// plans this action for a non-GitHub provider).
pub async fn update(
    config: &Config,
    settings: &GitHubActionsSecuritySettings,
) -> Result<(), RepoLensError> {
    let provider = crate::providers::for_config(config).ok_or(RepoLensError::Provider(
        ProviderError::GitHubCliNotAvailable,
    ))?;

    let to_action_error = |e: RepoLensError, what: &str| match e {
        RepoLensError::Provider(_) | RepoLensError::Action(_) => e,
        other => RepoLensError::Action(ActionError::ExecutionFailed {
            message: format!("Failed to update {what}: {other}"),
        }),
    };

    if settings.secret_scanning.is_some() || settings.secret_scanning_push_protection.is_some() {
        provider
            .set_secret_scanning(
                settings.secret_scanning,
                settings.secret_scanning_push_protection,
            )
            .map_err(|e| to_action_error(e, "secret scanning / push protection"))?;
    }

    if let Some(allowed) = &settings.allowed_actions {
        provider
            .set_actions_permissions(Some(allowed.as_str()))
            .map_err(|e| to_action_error(e, "Actions permissions"))?;
    }

    if let Some(perm) = &settings.default_workflow_permissions {
        provider
            .set_actions_workflow_permissions(Some(perm.as_str()))
            .map_err(|e| to_action_error(e, "default workflow permissions"))?;
    }

    if let Some(require_approval) = settings.require_fork_pr_approval {
        provider
            .set_fork_pr_workflows_policy(require_approval)
            .map_err(|e| to_action_error(e, "fork pull request workflow approval policy"))?;
    }

    Ok(())
}
