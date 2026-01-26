//! GitHub repository settings management

use anyhow::{bail, Context, Result};
use std::process::Command;

use super::plan::GitHubRepoSettings;
use crate::utils::prerequisites::{get_repo_info, is_gh_available};

/// Update GitHub repository settings
pub async fn update(settings: &GitHubRepoSettings) -> Result<()> {
    // Check if gh CLI is available
    if !is_gh_available() {
        bail!("GitHub CLI (gh) is not installed or not authenticated.");
    }

    // Get repository info
    let repo = get_repo_info()?;

    // Update repository settings
    let mut args = vec!["repo", "edit"];

    if let Some(true) = settings.enable_discussions {
        args.push("--enable-discussions");
    }

    if let Some(false) = settings.enable_wiki {
        args.push("--enable-wiki=false");
    }

    // Execute repository edit
    if args.len() > 2 {
        let output = Command::new("gh")
            .args(&args)
            .output()
            .context("Failed to update repository settings")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!("Could not update some repository settings: {}", stderr);
        }
    }

    // Enable vulnerability alerts
    if let Some(true) = settings.enable_vulnerability_alerts {
        let output = Command::new("gh")
            .args([
                "api",
                &format!("repos/{}/vulnerability-alerts", repo),
                "--method",
                "PUT",
            ])
            .output()
            .context("Failed to enable vulnerability alerts")?;

        if !output.status.success() {
            tracing::warn!(
                "Could not enable vulnerability alerts (may require specific permissions)"
            );
        }
    }

    // Enable automated security fixes
    if let Some(true) = settings.enable_automated_security_fixes {
        let output = Command::new("gh")
            .args([
                "api",
                &format!("repos/{}/automated-security-fixes", repo),
                "--method",
                "PUT",
            ])
            .output()
            .context("Failed to enable automated security fixes")?;

        if !output.status.success() {
            tracing::warn!(
                "Could not enable automated security fixes (may require specific permissions)"
            );
        }
    }

    Ok(())
}
