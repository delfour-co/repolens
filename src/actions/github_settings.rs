//! GitHub repository settings management

use anyhow::{Context, Result, bail};
use std::process::Command;

use super::plan::GitHubRepoSettings;

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
                "--method", "PUT",
            ])
            .output()
            .context("Failed to enable vulnerability alerts")?;

        if !output.status.success() {
            tracing::warn!("Could not enable vulnerability alerts (may require specific permissions)");
        }
    }

    // Enable automated security fixes
    if let Some(true) = settings.enable_automated_security_fixes {
        let output = Command::new("gh")
            .args([
                "api",
                &format!("repos/{}/automated-security-fixes", repo),
                "--method", "PUT",
            ])
            .output()
            .context("Failed to enable automated security fixes")?;

        if !output.status.success() {
            tracing::warn!("Could not enable automated security fixes (may require specific permissions)");
        }
    }

    Ok(())
}

/// Check if gh CLI is available and authenticated
fn is_gh_available() -> bool {
    Command::new("gh")
        .args(["auth", "status"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Get repository info (owner/name)
fn get_repo_info() -> Result<String> {
    let output = Command::new("gh")
        .args(["repo", "view", "--json", "nameWithOwner", "-q", ".nameWithOwner"])
        .output()
        .context("Failed to get repository info")?;

    if !output.status.success() {
        bail!("Failed to get repository info");
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
