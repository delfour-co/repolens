//! Branch protection configuration via GitHub API

use crate::error::{ActionError, ProviderError, RepoLensError};
use serde_json::json;
use std::io::Write;
use std::process::{Command, Stdio};

use super::plan::BranchProtectionSettings;
use crate::utils::prerequisites::{get_repo_info, is_gh_available};

/// Configure branch protection for a branch
pub async fn configure(
    branch: &str,
    settings: &BranchProtectionSettings,
) -> Result<(), RepoLensError> {
    // Check if gh CLI is available
    if !is_gh_available() {
        return Err(RepoLensError::Provider(
            ProviderError::GitHubCliNotAvailable,
        ));
    }

    // Get repository info
    let repo = get_repo_info().map_err(|e| {
        RepoLensError::Action(ActionError::ExecutionFailed {
            message: format!("Failed to get repository info: {}", e),
        })
    })?;

    // Build the JSON payload according to GitHub API specification
    let mut payload = json!({
        "enforce_admins": settings.enforce_admins,
        "required_linear_history": settings.require_linear_history,
        "allow_force_pushes": !settings.block_force_push,
        "allow_deletions": !settings.block_deletions,
        "required_conversation_resolution": settings.require_conversation_resolution,
        "restrictions": null,
    });

    // Handle required_status_checks
    if settings.require_status_checks {
        payload["required_status_checks"] = json!({
            "strict": true,
            "contexts": []
        });
    } else {
        payload["required_status_checks"] = json!(null);
    }

    // Handle required_pull_request_reviews
    if settings.required_approvals > 0 {
        payload["required_pull_request_reviews"] = json!({
            "required_approving_review_count": settings.required_approvals,
            "dismiss_stale_reviews": true
        });
    } else {
        payload["required_pull_request_reviews"] = json!(null);
    }

    // Execute the API call using --input to pass JSON via stdin
    let mut child = Command::new("gh")
        .args([
            "api",
            &format!("repos/{}/branches/{}/protection", repo, branch),
            "--method",
            "PUT",
            "--input",
            "-",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| {
            RepoLensError::Provider(ProviderError::CommandFailed {
                command: format!("gh api repos/{}/branches/{}/protection", repo, branch),
            })
        })?;

    // Write JSON to stdin
    let json_str = serde_json::to_string(&payload).map_err(|e| {
        RepoLensError::Action(ActionError::ExecutionFailed {
            message: format!("Failed to serialize branch protection payload: {}", e),
        })
    })?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(json_str.as_bytes()).map_err(|e| {
            RepoLensError::Action(ActionError::ExecutionFailed {
                message: format!("Failed to write to gh CLI stdin: {}", e),
            })
        })?;
    }

    let output = child.wait_with_output().map_err(|_| {
        RepoLensError::Provider(ProviderError::CommandFailed {
            command: format!("gh api repos/{}/branches/{}/protection", repo, branch),
        })
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Check for common errors
        if stderr.contains("Resource not accessible") {
            return Err(RepoLensError::Action(ActionError::ExecutionFailed {
                message: "Cannot configure branch protection. This may require admin access or \
                the repository may not support this feature (e.g., free private repos)."
                    .to_string(),
            }));
        }

        return Err(RepoLensError::Action(ActionError::ExecutionFailed {
            message: format!("Failed to configure branch protection: {}", stderr),
        }));
    }

    // Configure signed commits if required (separate API call)
    if settings.require_signed_commits {
        let output = Command::new("gh")
            .args([
                "api",
                &format!(
                    "repos/{}/branches/{}/protection/required_signatures",
                    repo, branch
                ),
                "--method",
                "POST",
            ])
            .output()
            .map_err(|_| {
                RepoLensError::Provider(ProviderError::CommandFailed {
                    command: format!(
                        "gh api repos/{}/branches/{}/protection/required_signatures",
                        repo, branch
                    ),
                })
            })?;

        if !output.status.success() {
            // Non-fatal: signed commits may not be available
            tracing::warn!("Could not enable signed commits requirement (may require GitHub Pro)");
        }
    }

    Ok(())
}
