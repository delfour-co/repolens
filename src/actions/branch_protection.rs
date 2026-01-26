//! Branch protection configuration via GitHub API

use anyhow::{bail, Context, Result};
use std::process::Command;

use super::plan::BranchProtectionSettings;
use crate::utils::prerequisites::{get_repo_info, is_gh_available};

/// Configure branch protection for a branch
pub async fn configure(branch: &str, settings: &BranchProtectionSettings) -> Result<()> {
    // Check if gh CLI is available
    if !is_gh_available() {
        bail!("GitHub CLI (gh) is not installed or not authenticated. Please install and run 'gh auth login'.");
    }

    // Get repository info
    let repo = get_repo_info()?;

    // Build the API request
    let required_pr_reviews = if settings.required_approvals > 0 {
        format!(
            r#"{{"required_approving_review_count":{},"dismiss_stale_reviews":true}}"#,
            settings.required_approvals
        )
    } else {
        "null".to_string()
    };

    let required_status_checks = if settings.require_status_checks {
        r#"{"strict":true,"contexts":[]}"#.to_string()
    } else {
        "null".to_string()
    };

    // Execute the API call
    let output = Command::new("gh")
        .args([
            "api",
            &format!("repos/{}/branches/{}/protection", repo, branch),
            "--method",
            "PUT",
            "--field",
            &format!("required_status_checks={}", required_status_checks),
            "--field",
            &format!("enforce_admins={}", settings.enforce_admins),
            "--field",
            &format!("required_pull_request_reviews={}", required_pr_reviews),
            "--field",
            "restrictions=null",
            "--field",
            &format!(
                "required_linear_history={}",
                settings.require_linear_history
            ),
            "--field",
            &format!("allow_force_pushes={}", !settings.block_force_push),
            "--field",
            &format!("allow_deletions={}", !settings.block_deletions),
            "--field",
            &format!(
                "required_conversation_resolution={}",
                settings.require_conversation_resolution
            ),
        ])
        .output()
        .context("Failed to execute gh command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Check for common errors
        if stderr.contains("Resource not accessible") {
            bail!(
                "Cannot configure branch protection. This may require admin access or \
                the repository may not support this feature (e.g., free private repos)."
            );
        }

        bail!("Failed to configure branch protection: {}", stderr);
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
            .context("Failed to enable signed commits requirement")?;

        if !output.status.success() {
            // Non-fatal: signed commits may not be available
            tracing::warn!("Could not enable signed commits requirement (may require GitHub Pro)");
        }
    }

    Ok(())
}
